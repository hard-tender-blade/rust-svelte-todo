<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { useQueryClient, createQueries } from '@tanstack/svelte-query';
	import {
		createListTenants,
		getGetTenantFansCountQueryOptions
	} from '$lib/api/generated/tenants/tenants';
	import { createListBillingEntries } from '$lib/api/generated/billing/billing';
	import {
		createUpsertTenantNote,
		createDeleteTenantNote,
		getGetTenantNoteQueryKey,
		getGetTenantNoteQueryOptions
	} from '$lib/api/generated/tenant-notes/tenant-notes';
	import type { BillingEntry, TenantResponse } from '$lib/api/generated/rustSvelteTodo.schemas';
	import { BillingCondition } from '$lib/api/generated/rustSvelteTodo.schemas';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { ContextMenu as ContextMenuPrimitive } from 'bits-ui';
	import CheckIcon from '@lucide/svelte/icons/check';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import SlidersHorizontalIcon from '@lucide/svelte/icons/sliders-horizontal';
	import InvoiceCell from './InvoiceCell.svelte';

	const queryClient = useQueryClient();

	// ── Tenants ────────────────────────────────────────────────────────────────
	const tenantsQuery = createListTenants();
	const allTenants = $derived(tenantsQuery.data?.status === 200 ? tenantsQuery.data.data : []);

	let search = $state('');

	function normalize(s: string) {
		return s
			.normalize('NFD')
			.replace(/\p{Diacritic}/gu, '')
			.toLowerCase();
	}

	const tenants = $derived(
		search.trim()
			? allTenants.filter((t) => normalize(t.name).includes(normalize(search.trim())))
			: allTenants
	);

	function planVariant(plan: string | null | undefined): 'default' | 'secondary' | 'outline' {
		if (plan === 'PRO') return 'default';
		if (plan === 'BASIC') return 'secondary';
		return 'outline';
	}

	type Segment = { text: string; match: boolean };

	function highlight(name: string, query: string): Segment[] {
		const q = normalize(query.trim());
		if (!q) return [{ text: name, match: false }];
		const normName = normalize(name);
		const segments: Segment[] = [];
		let i = 0;
		while (i < name.length) {
			const idx = normName.indexOf(q, i);
			if (idx === -1) {
				segments.push({ text: name.slice(i), match: false });
				break;
			}
			if (idx > i) segments.push({ text: name.slice(i, idx), match: false });
			segments.push({ text: name.slice(idx, idx + q.length), match: true });
			i = idx + q.length;
		}
		return segments;
	}

	// ── Parallel note fetch for every tenant ───────────────────────────────────
	const noteQueries = createQueries(() => ({
		queries: allTenants.map((t) => getGetTenantNoteQueryOptions(t.id))
	}));

	// ── Billing entries (used for scope calculation) ───────────────────────────
	const billingQuery = createListBillingEntries();
	const billingEntries = $derived(billingQuery.data?.status === 200 ? billingQuery.data.data : []);

	function getScopeBillingEntry(fans: number, entries: BillingEntry[]): BillingEntry | null {
		if (entries.length === 0) return null;
		const lessThan = entries
			.filter((e) => e.condition === BillingCondition.less_than)
			.sort((a, b) => a.fans_count - b.fans_count);
		const match = lessThan.find((e) => e.fans_count > fans);
		if (match) return match;
		return entries.reduce((max, e) => (e.fans_count > max.fans_count ? e : max), entries[0]);
	}

	function fillColor(value: number, max: number): string {
		const ratio = Math.min(value / max, 1);
		const hue = 120 * Math.sin(ratio * Math.PI);
		return `hsl(${hue}, 72%, 42%)`;
	}

	function getScopeThreshold(fans: number, entries: BillingEntry[]): number | null {
		return getScopeBillingEntry(fans, entries)?.fans_count ?? null;
	}

	function getExpectedPrice(
		fans: number | null,
		plan: string | null | undefined,
		entries: BillingEntry[]
	): number | null {
		if (fans === null || !plan) return null;
		const entry = getScopeBillingEntry(fans, entries);
		if (!entry) return null;
		const p = plan.toUpperCase();
		if (p === 'BASIC') return entry.basic_plan_price ?? null;
		if (p === 'STANDARD') return entry.standard_plan_price ?? null;
		if (p === 'PRO' || p === 'PREMIUM') return entry.premium_plan_price ?? null;
		return null;
	}

	// ── Parallel fans count fetch for every tenant ─────────────────────────────
	const fansQueries = createQueries(() => ({
		queries: allTenants.map((t) => getGetTenantFansCountQueryOptions(t.id))
	}));

	function getFansCount(tenantIndex: number): number | null {
		const q = fansQueries[tenantIndex];
		return q?.data?.status === 200 ? q.data.data.fans_count : null;
	}

	function getNoteText(tenantIndex: number): string | null {
		const q = noteQueries[tenantIndex];
		return q?.data?.status === 200 ? (q.data.data.note ?? null) : null;
	}

	// ── Edit note ──────────────────────────────────────────────────────────────
	let editOpen = $state(false);
	let editingTenant = $state<TenantResponse | null>(null);
	let editNoteText = $state('');

	function openEdit(tenant: TenantResponse, tenantIndex: number) {
		editingTenant = tenant;
		editNoteText = getNoteText(tenantIndex) ?? '';
		editOpen = true;
	}

	const upsertMutation = createUpsertTenantNote(() => ({
		mutation: {
			onSuccess: () => {
				queryClient.invalidateQueries({
					queryKey: getGetTenantNoteQueryKey(editingTenant!.id)
				});
				toast.success('Note saved');
				editOpen = false;
				editingTenant = null;
			},
			onError: () => toast.error('Failed to save note')
		}
	}));

	function submitEdit(e: SubmitEvent) {
		e.preventDefault();
		if (!editingTenant) return;
		upsertMutation.mutate({
			mongoId: editingTenant.id,
			data: { note: editNoteText.trim() || null }
		});
	}

	// ── Delete note ────────────────────────────────────────────────────────────
	let deleteOpen = $state(false);
	let deletingTenant = $state<TenantResponse | null>(null);

	function openDelete(tenant: TenantResponse) {
		deletingTenant = tenant;
		deleteOpen = true;
	}

	const deleteMutation = createDeleteTenantNote(() => ({
		mutation: {
			onSuccess: () => {
				if (deletingTenant) {
					queryClient.invalidateQueries({
						queryKey: getGetTenantNoteQueryKey(deletingTenant.id)
					});
				}
				toast.success('Note deleted');
				deleteOpen = false;
				deletingTenant = null;
			},
			onError: () => toast.error('Failed to delete note')
		}
	}));

	// ── Column visibility ──────────────────────────────────────────────────────
	type ColKey = 'index' | 'plan' | 'fans' | 'fill' | 'price' | 'note';

	const allColOptions: { key: ColKey; label: string }[] = [
		{ key: 'index', label: '#' },
		{ key: 'plan', label: 'Plan' },
		{ key: 'fans', label: 'Fans / Scope' },
		{ key: 'fill', label: 'Fill' },
		{ key: 'price', label: 'Expected price' },
		{ key: 'note', label: 'Note' }
	];

	let selectedColKeys = $state(new Set<ColKey>(['index', 'plan', 'fans', 'fill', 'price', 'note']));

	function toggleCol(key: ColKey) {
		const next = new Set(selectedColKeys);
		if (next.has(key)) next.delete(key);
		else next.add(key);
		selectedColKeys = next;
	}

	function hasCol(key: ColKey): boolean {
		return selectedColKeys.has(key);
	}

	// ── Month selector ─────────────────────────────────────────────────────────
	type MonthOption = { year: number; month: number; label: string; key: string };

	function buildMonthOptions(): MonthOption[] {
		const now = new Date();
		const opts: MonthOption[] = [];
		for (let i = -6; i <= 3; i++) {
			const d = new Date(now.getFullYear(), now.getMonth() + i, 1);
			const year = d.getFullYear();
			const month = d.getMonth() + 1;
			opts.push({
				year,
				month,
				label: `${String(month).padStart(2, '0')}.${year}`,
				key: `${year}-${String(month).padStart(2, '0')}`
			});
		}
		return opts;
	}

	// allMonthOptions[6] = current month, [5] = prev, [7] = next
	const allMonthOptions = buildMonthOptions();
	const currentMonthKey = allMonthOptions[6].key;

	let selectedMonthKeys = $state(
		new Set([allMonthOptions[5].key, allMonthOptions[6].key, allMonthOptions[7].key])
	);

	function toggleMonth(key: string) {
		const next = new Set(selectedMonthKeys);
		if (next.has(key)) {
			if (next.size > 1) next.delete(key);
		} else {
			next.add(key);
		}
		selectedMonthKeys = next;
	}

	// Preserves chronological order from allMonthOptions
	const selectedMonths = $derived(allMonthOptions.filter((m) => selectedMonthKeys.has(m.key)));

	// 2 fixed (logo + name) + optional cols + invoice month cols
	const totalCols = $derived(2 + selectedColKeys.size + selectedMonths.length);
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between gap-3">
		<div>
			<h1 class="text-2xl font-semibold tracking-tight">Invoices</h1>
			<p class="text-sm text-muted-foreground">Admin notes attached to individual tenants.</p>
		</div>
		<div class="flex items-center gap-2">
				<!-- Column visibility selector -->
				<DropdownMenu.Root>
					<DropdownMenu.Trigger>
						{#snippet child({ props })}
							<Button {...props} variant="outline" size="sm" class="gap-2">
								<SlidersHorizontalIcon class="size-4" />
								Columns ({selectedColKeys.size})
							</Button>
						{/snippet}
					</DropdownMenu.Trigger>
					<DropdownMenu.Content align="end" class="w-48">
						<DropdownMenu.Label class="text-xs text-muted-foreground">
							Visible columns
						</DropdownMenu.Label>
						<DropdownMenu.Separator />
						{#each allColOptions as opt (opt.key)}
							<DropdownMenu.Item class="gap-2" onclick={() => toggleCol(opt.key)}>
								<CheckIcon
									class="size-4 shrink-0 transition-opacity {selectedColKeys.has(opt.key)
										? 'opacity-100'
										: 'opacity-0'}"
								/>
								{opt.label}
							</DropdownMenu.Item>
						{/each}
					</DropdownMenu.Content>
				</DropdownMenu.Root>

				<!-- Month selector -->
				<DropdownMenu.Root>
				<DropdownMenu.Trigger>
					{#snippet child({ props })}
						<Button {...props} variant="outline" size="sm" class="gap-2">
							<CalendarIcon class="size-4" />
							Months ({selectedMonthKeys.size})
						</Button>
					{/snippet}
				</DropdownMenu.Trigger>
				<DropdownMenu.Content align="end" class="w-44">
					<DropdownMenu.Label class="text-xs text-muted-foreground">
						Invoice months
					</DropdownMenu.Label>
					<DropdownMenu.Separator />
					{#each allMonthOptions as opt (opt.key)}
						<DropdownMenu.Item class="gap-2" onclick={() => toggleMonth(opt.key)}>
							<CheckIcon
								class="size-4 shrink-0 transition-opacity {selectedMonthKeys.has(opt.key)
									? 'opacity-100'
									: 'opacity-0'}"
							/>
							<span class={opt.key === currentMonthKey ? 'font-medium' : ''}>{opt.label}</span>
						</DropdownMenu.Item>
					{/each}
				</DropdownMenu.Content>
			</DropdownMenu.Root>

			<Input bind:value={search} placeholder="Search by name…" class="w-64" />
		</div>
	</div>

	{#if tenantsQuery.isPending}
		<div class="text-sm text-muted-foreground">Loading…</div>
	{:else if tenantsQuery.isError}
		<div class="text-sm text-destructive">Failed to load tenants.</div>
	{:else}
		<div class="overflow-x-auto rounded-lg border">
			<Table.Root class="min-w-max">
				<Table.Header>
					<Table.Row>
						{#if hasCol('index')}<Table.Head class="w-12 text-center">#</Table.Head>{/if}
						<Table.Head class="w-12"></Table.Head>
						<Table.Head class="w-56">Name</Table.Head>
						{#if hasCol('plan')}<Table.Head class="w-24">Plan</Table.Head>{/if}
						{#if hasCol('fans')}<Table.Head class="w-44 text-right">Fans / Scope</Table.Head>{/if}
						{#if hasCol('fill')}<Table.Head class="w-32 min-w-32">Fill</Table.Head>{/if}
						{#if hasCol('price')}<Table.Head class="w-32 text-right">Expected price</Table.Head>{/if}
						{#if hasCol('note')}<Table.Head>Note</Table.Head>{/if}
						<!-- One column per selected month, right side -->
						{#each selectedMonths as month (month.key)}
							<Table.Head
								class="min-w-[130px] text-center text-xs font-medium {month.key ===
								currentMonthKey
									? 'bg-muted/40'
									: ''}"
							>
								{month.label}
							</Table.Head>
						{/each}
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each tenants as tenant, i (tenant.id)}
						{@const ai = allTenants.findIndex((t) => t.id === tenant.id)}
						{@const note = getNoteText(ai)}
						{@const isPending = noteQueries[ai]?.isPending ?? true}
						<Table.Row>
							{#if hasCol('index')}
								<Table.Cell class="text-center text-xs text-muted-foreground">
									{i + 1}
								</Table.Cell>
							{/if}
							<Table.Cell>
								{#if tenant.club_logo}
									<img
										src={tenant.club_logo}
										alt={tenant.name}
										class="h-8 w-8 rounded-full object-contain"
									/>
								{:else}
									<div
										class="flex h-8 w-8 items-center justify-center rounded-full bg-muted text-xs font-semibold text-muted-foreground"
									>
										{tenant.name.slice(0, 2).toUpperCase()}
									</div>
								{/if}
							</Table.Cell>
							<Table.Cell class="font-medium">
								{#each highlight(tenant.name, search) as seg, segIndex (segIndex)}
									{#if seg.match}
										<mark
											class="rounded bg-yellow-300/60 px-0.5 text-inherit dark:bg-yellow-500/40"
											>{seg.text}</mark
										>
									{:else}
										{seg.text}
									{/if}
								{/each}
							</Table.Cell>
							{#if hasCol('plan')}
								<Table.Cell>
									{#if tenant.plan}
										<Badge variant={planVariant(tenant.plan)}>{tenant.plan}</Badge>
									{:else}
										<span class="text-muted-foreground">—</span>
									{/if}
								</Table.Cell>
							{/if}
							{#if hasCol('fans')}
								<Table.Cell class="text-right font-mono text-muted-foreground tabular-nums">
									{#if fansQueries[ai]?.isPending ?? true}
										<span class="text-xs text-muted-foreground/50">…</span>
									{:else}
										{@const fans = getFansCount(ai)}
										{@const threshold =
											fans !== null ? getScopeThreshold(fans, billingEntries) : null}
										{#if fans !== null && threshold !== null}
											{fans.toLocaleString()} / {threshold.toLocaleString()}
										{:else}
											<span class="text-muted-foreground">—</span>
										{/if}
									{/if}
								</Table.Cell>
							{/if}
							{#if hasCol('fill')}
								<Table.Cell class="w-32 min-w-32 pr-4">
									{#if fansQueries[ai]?.isPending ?? true}
										<div class="h-3 w-full animate-pulse rounded-full bg-muted"></div>
									{:else}
										{@const fans = getFansCount(ai)}
										{@const threshold =
											fans !== null ? getScopeThreshold(fans, billingEntries) : null}
										{#if fans !== null && threshold !== null}
											<div class="relative h-2 w-full overflow-hidden rounded-full bg-muted">
												<div
													class="h-full rounded-full transition-all"
													style="width: {Math.min(
														(fans / threshold) * 100,
														100
													)}%; background-color: {fillColor(fans, threshold)};"
												></div>
											</div>
										{:else}
											<span class="text-xs text-muted-foreground">—</span>
										{/if}
									{/if}
								</Table.Cell>
							{/if}
							{#if hasCol('price')}
								<Table.Cell class="text-right font-mono tabular-nums">
									{#if fansQueries[ai]?.isPending ?? true}
										<span class="text-xs text-muted-foreground/50">…</span>
									{:else}
										{@const price = getExpectedPrice(
											getFansCount(ai),
											tenant.plan,
											billingEntries
										)}
										{#if price !== null}
											{price.toLocaleString()}
										{:else}
											<span class="text-muted-foreground">—</span>
										{/if}
									{/if}
								</Table.Cell>
							{/if}

							<!-- Note cell — right-click for context menu -->
							{#if hasCol('note')}
							<ContextMenu.Root>
								<ContextMenuPrimitive.Trigger>
									{#snippet child({ props })}
										<Table.Cell {...props} class="cursor-context-menu">
											{#if isPending}
												<span class="text-xs text-muted-foreground/50">Loading…</span>
											{:else if note}
												<span class="line-clamp-2 text-sm">{note}</span>
											{:else}
												<span class="text-xs text-muted-foreground/40">—</span>
											{/if}
										</Table.Cell>
									{/snippet}
								</ContextMenuPrimitive.Trigger>
								<ContextMenu.Content>
									<ContextMenu.Label class="text-xs text-muted-foreground">
										{tenant.name}
									</ContextMenu.Label>
									<ContextMenu.Separator />
									<ContextMenu.Item onclick={() => openEdit(tenant, ai)}>
										Edit Note
									</ContextMenu.Item>
									<ContextMenu.Separator />
									<ContextMenu.Item
										class="text-destructive focus:text-destructive"
										onclick={() => openDelete(tenant)}
									>
										Delete Note
									</ContextMenu.Item>
								</ContextMenu.Content>
							</ContextMenu.Root>
							{/if}

							<!-- Invoice month cells — each owns its own query + editing state -->
							{#each selectedMonths as month (month.key)}
								<Table.Cell
									class="min-w-[130px] p-0 {month.key === currentMonthKey
										? 'bg-muted/20'
										: ''}"
								>
									<InvoiceCell
										tenantId={tenant.id}
										year={month.year}
										month={month.month}
									/>
								</Table.Cell>
							{/each}
						</Table.Row>
					{:else}
						<Table.Row>
							<Table.Cell class="py-8 text-center text-muted-foreground" colspan={totalCols}>
								No tenants found.
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</div>
	{/if}
</div>

<!-- Edit Note Dialog -->
<Dialog.Root bind:open={editOpen}>
	<Dialog.Content class="sm:max-w-[480px]">
		<Dialog.Header>
			<Dialog.Title>Edit Note</Dialog.Title>
			<Dialog.Description>Note for <strong>{editingTenant?.name}</strong>.</Dialog.Description>
		</Dialog.Header>
		<form class="space-y-4 py-2" onsubmit={submitEdit}>
			<textarea
				bind:value={editNoteText}
				placeholder="Write a note…"
				rows={6}
				class="w-full resize-y rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
			></textarea>
			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (editOpen = false)}>Cancel</Button>
				<Button type="submit" disabled={upsertMutation.isPending}>
					{upsertMutation.isPending ? 'Saving…' : 'Save'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Delete Confirm Dialog -->
<Dialog.Root bind:open={deleteOpen}>
	<Dialog.Content class="sm:max-w-[400px]">
		<Dialog.Header>
			<Dialog.Title>Delete Note</Dialog.Title>
			<Dialog.Description>
				Are you sure you want to delete the note for
				<strong>{deletingTenant?.name}</strong>? This cannot be undone.
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (deleteOpen = false)}>Cancel</Button>
			<Button
				variant="destructive"
				disabled={deleteMutation.isPending}
				onclick={() => {
					if (deletingTenant) deleteMutation.mutate({ mongoId: deletingTenant.id });
				}}
			>
				{deleteMutation.isPending ? 'Deleting…' : 'Delete'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
