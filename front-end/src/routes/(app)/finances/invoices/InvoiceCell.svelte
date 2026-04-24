<script lang="ts">
	import { useQueryClient } from '@tanstack/svelte-query';
	import { toast } from 'svelte-sonner';
	import {
		createListInvoicingEntriesForMonth,
		getListInvoicingEntriesForMonthQueryKey,
		createCreateInvoicingEntry,
		createUpsertInvoicingEntry
	} from '$lib/api/generated/admin-invoicing/admin-invoicing';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import { ContextMenu as ContextMenuPrimitive, Popover as PopoverPrimitive } from 'bits-ui';
	import MessageSquareIcon from '@lucide/svelte/icons/message-square';

	let {
		tenantId,
		year,
		month
	}: {
		tenantId: string;
		year: number;
		month: number;
	} = $props();

	const queryClient = useQueryClient();

	const monthQuery = createListInvoicingEntriesForMonth(
		() => tenantId,
		() => ({ year, month })
	);

	const entry = $derived(
		monthQuery.data?.status === 200 && monthQuery.data.data.length > 0
			? monthQuery.data.data[0]
			: null
	);

	// ── Price editing ──────────────────────────────────────────────────────────
	let editing = $state(false);
	let editPrice = $state('');

	function startEdit() {
		editPrice = entry !== null ? String(entry.price) : '';
		editing = true;
		hoverOpen = false;
	}

	function cancelEdit() {
		editing = false;
	}

	const createMutation = createCreateInvoicingEntry(() => ({
		mutation: {
			onSuccess: () => {
				queryClient.invalidateQueries({
					queryKey: getListInvoicingEntriesForMonthQueryKey(tenantId, { year, month })
				});
				toast.success('Invoice saved');
				editing = false;
			},
			onError: () => toast.error('Failed to save invoice')
		}
	}));

	const upsertMutation = createUpsertInvoicingEntry(() => ({
		mutation: {
			onSuccess: () => {
				queryClient.invalidateQueries({
					queryKey: getListInvoicingEntriesForMonthQueryKey(tenantId, { year, month })
				});
				toast.success('Invoice saved');
				editing = false;
			},
			onError: () => toast.error('Failed to save invoice')
		}
	}));

	const isSaving = $derived(createMutation.isPending || upsertMutation.isPending);

	function submit() {
		if (isSaving) return;
		const price = parseInt(editPrice.trim(), 10);
		if (!editPrice.trim() || isNaN(price)) {
			cancelEdit();
			return;
		}
		const date = `${year}-${String(month).padStart(2, '0')}-01`;
		// Preserve existing note when only editing price
		const payload = { mongo_id: tenantId, date, price, note: entry?.note ?? null };
		if (entry) {
			upsertMutation.mutate({ id: entry.id, data: payload });
		} else {
			createMutation.mutate({ data: payload });
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			submit();
		} else if (e.key === 'Escape') {
			cancelEdit();
		}
	}

	function autoFocus(el: HTMLElement) {
		el.focus();
	}

	// ── Hover popover ──────────────────────────────────────────────────────────
	let hoverOpen = $state(false);

	// ── Note dialog (right-click) ──────────────────────────────────────────────
	let noteDialogOpen = $state(false);
	let noteDialogText = $state('');
	let deleteNoteDialogOpen = $state(false);

	function openNoteDialog() {
		noteDialogText = entry?.note ?? '';
		noteDialogOpen = true;
	}

	const noteUpsertMutation = createUpsertInvoicingEntry(() => ({
		mutation: {
			onSuccess: () => {
				queryClient.invalidateQueries({
					queryKey: getListInvoicingEntriesForMonthQueryKey(tenantId, { year, month })
				});
				toast.success('Note saved');
				noteDialogOpen = false;
				deleteNoteDialogOpen = false;
			},
			onError: () => toast.error('Failed to save note')
		}
	}));

	function submitNoteDialog(e: SubmitEvent) {
		e.preventDefault();
		if (!entry || noteUpsertMutation.isPending) return;
		noteUpsertMutation.mutate({
			id: entry.id,
			data: {
				mongo_id: tenantId,
				date: entry.date,
				price: entry.price,
				note: noteDialogText.trim() || null
			}
		});
	}

	function deleteNote() {
		if (!entry || noteUpsertMutation.isPending) return;
		noteUpsertMutation.mutate({
			id: entry.id,
			data: { mongo_id: tenantId, date: entry.date, price: entry.price, note: null }
		});
	}
</script>

<div class="w-full">
{#if editing}
	<!--
		Same min-h and padding as the display button so the cell doesn't shift.
		Input is transparent/borderless — only a subtle underline indicates edit mode.
	-->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="flex min-h-12 w-full items-center px-3 py-2"
		onmousedown={(e) => e.stopPropagation()}
	>
		<input
			type="text"
			inputmode="numeric"
			bind:value={editPrice}
			placeholder="0"
			disabled={isSaving}
			class="w-full bg-transparent font-mono text-sm tabular-nums border-b border-border pb-px focus:border-primary focus:outline-none placeholder:text-muted-foreground/30 disabled:opacity-50"
			onkeydown={handleKeydown}
			use:autoFocus
		/>
	</div>
{:else if monthQuery.isPending}
	<div class="flex h-12 items-center justify-center px-3">
		<div class="h-2 w-12 animate-pulse rounded-full bg-muted"></div>
	</div>
{:else}
	<!--
		Popover.Root is fully controlled (onOpenChange no-op) so only hover events drive it.
		PopoverPrimitive.Trigger is used directly (bits-ui native) to get {#snippet child}
		support without fighting the shadcn wrapper, letting us nest both triggers on one button.
	-->
	<Popover.Root open={entry?.note ? hoverOpen : false} onOpenChange={() => {}}>
		<PopoverPrimitive.Trigger>
			{#snippet child({ props: popProps })}
				<ContextMenu.Root>
					<ContextMenuPrimitive.Trigger>
						{#snippet child({ props: ctxProps })}
							<button
								type="button"
								{...popProps}
								{...ctxProps}
								class="flex min-h-12 w-full items-center justify-between px-3 py-2 text-left transition-colors hover:bg-muted/40"
								onclick={(e) => {
									e.stopPropagation();
									startEdit();
								}}
								onmousedown={(e) => e.stopPropagation()}
								onmouseenter={() => {
									if (entry?.note) hoverOpen = true;
								}}
								onmouseleave={() => {
									hoverOpen = false;
								}}
							>
								{#if entry}
									<span class="font-mono text-sm tabular-nums"
										>{entry.price.toLocaleString()}</span
									>
									{#if entry.note}
										<MessageSquareIcon class="size-3.5 shrink-0 text-muted-foreground/60" />
									{/if}
								{:else}
									<span class="text-xs text-muted-foreground/30">—</span>
								{/if}
							</button>
						{/snippet}
					</ContextMenuPrimitive.Trigger>
					{#if entry}
						<ContextMenu.Content>
							<ContextMenu.Label class="text-xs text-muted-foreground">Invoice note</ContextMenu.Label>
							<ContextMenu.Separator />
							<ContextMenu.Item onclick={openNoteDialog}>Edit Note</ContextMenu.Item>
							{#if entry.note}
								<ContextMenu.Separator />
								<ContextMenu.Item
									class="text-destructive focus:text-destructive"
									onclick={() => (deleteNoteDialogOpen = true)}
								>
									Delete Note
								</ContextMenu.Item>
							{/if}
						</ContextMenu.Content>
					{/if}
				</ContextMenu.Root>
			{/snippet}
		</PopoverPrimitive.Trigger>

		{#if entry?.note}
			<Popover.Content
				side="top"
				sideOffset={6}
				class="w-auto max-w-[220px] p-2.5"
				interactOutsideBehavior="ignore"
			>
				<p class="text-xs leading-snug">{entry.note}</p>
			</Popover.Content>
		{/if}
	</Popover.Root>
{/if}
</div>

<!-- Edit Invoice Note Dialog -->
<Dialog.Root bind:open={noteDialogOpen}>
	<Dialog.Content class="sm:max-w-[480px]">
		<Dialog.Header>
			<Dialog.Title>Edit Invoice Note</Dialog.Title>
			<Dialog.Description>
				Note for invoice {String(month).padStart(2, '0')}.{year}.
			</Dialog.Description>
		</Dialog.Header>
		<form class="space-y-4 py-2" onsubmit={submitNoteDialog}>
			<textarea
				bind:value={noteDialogText}
				placeholder="Write a note…"
				rows={6}
				class="w-full resize-y rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
			></textarea>
			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (noteDialogOpen = false)}>
					Cancel
				</Button>
				<Button type="submit" disabled={noteUpsertMutation.isPending}>
					{noteUpsertMutation.isPending ? 'Saving…' : 'Save'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Delete Invoice Note Confirm Dialog -->
<Dialog.Root bind:open={deleteNoteDialogOpen}>
	<Dialog.Content class="sm:max-w-[400px]">
		<Dialog.Header>
			<Dialog.Title>Delete Note</Dialog.Title>
			<Dialog.Description>
				Remove the note from this invoice? The price entry will be kept.
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (deleteNoteDialogOpen = false)}>Cancel</Button>
			<Button variant="destructive" disabled={noteUpsertMutation.isPending} onclick={deleteNote}>
				{noteUpsertMutation.isPending ? 'Deleting…' : 'Delete'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
