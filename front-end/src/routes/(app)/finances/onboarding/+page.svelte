<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { useQueryClient } from '@tanstack/svelte-query';
	import { Combobox } from 'bits-ui';
	import {
		createListAllOnboardingEntries,
		createCreateOnboardingEntry,
		createUpsertOnboardingEntry,
		createDeleteOnboardingEntry,
		getListAllOnboardingEntriesQueryKey
	} from '$lib/api/generated/admin-onboarding/admin-onboarding';
	import { createListTenants } from '$lib/api/generated/tenants/tenants';
	import { Currency } from '$lib/api/generated/rustSvelteTodo.schemas';
	import type { OnboardingEntry } from '$lib/api/generated/rustSvelteTodo.schemas';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Checkbox } from '$lib/components/ui/checkbox/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';
	import { ContextMenu as ContextMenuPrimitive } from 'bits-ui';

	const queryClient = useQueryClient();

	const onboardingQuery = createListAllOnboardingEntries();
	const entries = $derived(
		onboardingQuery.data?.status === 200 ? onboardingQuery.data.data : []
	);

	const tenantsQuery = createListTenants();
	const allTenants = $derived(
		tenantsQuery.data?.status === 200 ? tenantsQuery.data.data : []
	);
	const tenantMap = $derived(() => {
		const map = new Map<string, { name: string; hostname: string; club_logo?: string | null }>();
		for (const t of allTenants) {
			map.set(t.id, { name: t.name, hostname: t.hostname, club_logo: t.club_logo });
		}
		return map;
	});

	// Items array for bits-ui Combobox filtering
	const tenantItems = $derived(
		allTenants.map((t) => ({ value: t.id, label: t.name }))
	);

	// ── Search ───────────────────────────────────────────────────────────────
	let search = $state('');

	function normalize(s: string) {
		return s
			.normalize('NFD')
			.replace(/\p{Diacritic}/gu, '')
			.toLowerCase();
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

	const filteredEntries = $derived(
		search.trim()
			? entries.filter((e) => {
					const name = tenantMap().get(e.mongo_id)?.name ?? e.mongo_id;
					return normalize(name).includes(normalize(search.trim()));
				})
			: entries
	);

	const CURRENCY_LABELS: Record<string, string> = {
		[Currency.czk]: 'CZK',
		[Currency.eur]: 'EUR'
	};

	function fmtDate(d: string | null | undefined): string {
		if (!d) return '—';
		return new Date(d).toLocaleDateString('cs-CZ');
	}

	function fmtPrice(price: number, currency: string): string {
		return `${price.toLocaleString('cs-CZ')} ${CURRENCY_LABELS[currency] ?? currency}`;
	}

	// ── Shared blank form ─────────────────────────────────────────────────────
	function blankForm() {
		return {
			mongo_id: '',
			tenantSearch: '', // tracks Combobox inputValue — excluded from payload
			date_training: '',
			paid: false,
			price: 0,
			currency: Currency.czk as string,
			invoiced: false,
			invoiced_date: '',
			business_module: false,
			fans_module: false,
			note: '',
			enigoo_involved: false
		};
	}

	function formToPayload(form: ReturnType<typeof blankForm>) {
		return {
			mongo_id: form.mongo_id,
			date_training: form.date_training || null,
			paid: form.paid,
			price: Number(form.price),
			currency: form.currency as (typeof Currency)[keyof typeof Currency],
			invoiced: form.invoiced,
			invoiced_date: form.invoiced_date || null,
			business_module: form.business_module,
			fans_module: form.fans_module,
			note: form.note || null,
			enigoo_involved: form.enigoo_involved
		};
	}

	// ── Add entry ─────────────────────────────────────────────────────────────
	let addOpen = $state(false);
	let addForm = $state(blankForm());
	let addError = $state('');

	const addMutation = createCreateOnboardingEntry(() => ({
		mutation: {
			onSuccess: (result) => {
				if (result.status !== 201) {
					const msg = (result.data as { error?: string })?.error ?? 'Failed to create onboarding entry';
					addError = msg;
					toast.error(msg);
					return;
				}
				queryClient.invalidateQueries({ queryKey: getListAllOnboardingEntriesQueryKey() });
				toast.success('Onboarding entry created');
				addOpen = false;
				addForm = blankForm();
				addError = '';
			},
			onError: (err: { error?: string }) => {
				addError = err.error ?? 'Failed to create onboarding entry';
				toast.error(err.error ?? 'Failed to create onboarding entry');
			}
		}
	}));

	function submitAdd(e: SubmitEvent) {
		e.preventDefault();
		addMutation.mutate({ data: formToPayload(addForm) });
	}

	// ── Edit entry ────────────────────────────────────────────────────────────
	let editOpen = $state(false);
	let editingEntry = $state<OnboardingEntry | null>(null);
	let editForm = $state(blankForm());
	let editError = $state('');

	function openEdit(entry: OnboardingEntry) {
		editingEntry = entry;
		editForm = {
			mongo_id: entry.mongo_id,
			tenantSearch: tenantMap().get(entry.mongo_id)?.name ?? '',
			date_training: entry.date_training ?? '',
			paid: entry.paid,
			price: entry.price,
			currency: entry.currency,
			invoiced: entry.invoiced,
			invoiced_date: entry.invoiced_date ?? '',
			business_module: entry.business_module,
			fans_module: entry.fans_module,
			note: entry.note ?? '',
			enigoo_involved: entry.enigoo_involved
		};
		editError = '';
		editOpen = true;
	}

	const editMutation = createUpsertOnboardingEntry(() => ({
		mutation: {
			onSuccess: (result) => {
				if (result.status !== 200) {
					const msg = (result.data as { error?: string })?.error ?? 'Failed to update onboarding entry';
					editError = msg;
					toast.error(msg);
					return;
				}
				queryClient.invalidateQueries({ queryKey: getListAllOnboardingEntriesQueryKey() });
				toast.success('Onboarding entry updated');
				editOpen = false;
				editingEntry = null;
				editError = '';
			},
			onError: (err: { error?: string }) => {
				editError = err.error ?? 'Failed to update onboarding entry';
				toast.error(err.error ?? 'Failed to update onboarding entry');
			}
		}
	}));

	function submitEdit(e: SubmitEvent) {
		e.preventDefault();
		if (!editingEntry) return;
		editMutation.mutate({ id: editingEntry.id, data: formToPayload(editForm) });
	}

	// ── Delete entry ──────────────────────────────────────────────────────────
	let deleteOpen = $state(false);
	let deletingEntry = $state<OnboardingEntry | null>(null);

	function openDelete(entry: OnboardingEntry) {
		deletingEntry = entry;
		deleteOpen = true;
	}

	const deleteMutation = createDeleteOnboardingEntry(() => ({
		mutation: {
			onSuccess: () => {
				queryClient.invalidateQueries({ queryKey: getListAllOnboardingEntriesQueryKey() });
				toast.success('Onboarding entry deleted');
				deleteOpen = false;
				deletingEntry = null;
			},
			onError: () => {
				toast.error('Failed to delete onboarding entry');
			}
		}
	}));
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-semibold tracking-tight">Onboarding</h1>
			<p class="text-sm text-muted-foreground">
				Track onboarding sessions and training entries per tenant.
			</p>
		</div>
		<div class="flex items-center gap-2">
			<Input class="w-56" placeholder="Search tenants…" bind:value={search} />
			<Button onclick={() => (addOpen = true)}>+ Add Entry</Button>
		</div>
	</div>

	<!-- Table -->
	{#if onboardingQuery.isPending}
		<div class="text-sm text-muted-foreground">Loading...</div>
	{:else if onboardingQuery.isError}
		<div class="text-sm text-destructive">Failed to load onboarding entries.</div>
	{:else}
		<div class="rounded-lg border">
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head class="w-10 text-center">#</Table.Head>
						<Table.Head class="w-52">Tenant</Table.Head>
						<Table.Head class="w-44">Host</Table.Head>
						<Table.Head class="w-32">Training Date</Table.Head>
						<Table.Head class="w-32 text-right">Price</Table.Head>
						<Table.Head class="w-20 text-center">Paid</Table.Head>
						<Table.Head class="w-20 text-center">Invoiced</Table.Head>
						<Table.Head class="w-32">Invoiced Date</Table.Head>
						<Table.Head class="w-24 text-center">Business</Table.Head>
						<Table.Head class="w-20 text-center">Fans</Table.Head>
						<Table.Head class="w-20 text-center">Enigoo</Table.Head>
						<Table.Head>Note</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each filteredEntries as entry, i (entry.id)}
						{@const tenant = tenantMap().get(entry.mongo_id)}
						<ContextMenu.Root>
							<ContextMenuPrimitive.Trigger>
								{#snippet child({ props })}
									<Table.Row {...props} class="cursor-context-menu select-none">
										<Table.Cell class="text-center text-muted-foreground">{i + 1}</Table.Cell>
										<Table.Cell>
											<div class="flex items-center gap-2.5">
												{#if tenant?.club_logo}
													<img
														src={tenant.club_logo}
														alt={tenant.name}
														class="h-7 w-7 shrink-0 rounded-full object-contain"
													/>
												{:else}
													<div
														class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-muted text-xs font-semibold text-muted-foreground"
													>
														{(tenant?.name ?? entry.mongo_id).slice(0, 2).toUpperCase()}
													</div>
												{/if}
												<span class="truncate text-sm font-medium">
													{#each highlight(tenant?.name ?? entry.mongo_id, search) as seg}
														{#if seg.match}
															<mark
																class="rounded bg-yellow-300/60 px-0.5 text-inherit dark:bg-yellow-500/40"
																>{seg.text}</mark
															>
														{:else}
															{seg.text}
														{/if}
													{/each}
												</span>
											</div>
										</Table.Cell>
										<Table.Cell class="font-mono text-xs text-muted-foreground">
											{tenant?.hostname ?? '—'}
										</Table.Cell>
										<Table.Cell class="tabular-nums">{fmtDate(entry.date_training)}</Table.Cell>
										<Table.Cell class="text-right font-mono tabular-nums">
											{fmtPrice(entry.price, entry.currency)}
										</Table.Cell>
										<Table.Cell class="text-center">
											{#if entry.paid}
												<Badge variant="default">Yes</Badge>
											{:else}
												<Badge variant="outline">No</Badge>
											{/if}
										</Table.Cell>
										<Table.Cell class="text-center">
											{#if entry.invoiced}
												<Badge variant="default">Yes</Badge>
											{:else}
												<Badge variant="outline">No</Badge>
											{/if}
										</Table.Cell>
										<Table.Cell class="tabular-nums text-muted-foreground">
											{fmtDate(entry.invoiced_date)}
										</Table.Cell>
										<Table.Cell class="text-center">
											{#if entry.business_module}
												<Badge variant="default">Yes</Badge>
											{:else}
												<Badge variant="outline">No</Badge>
											{/if}
										</Table.Cell>
										<Table.Cell class="text-center">
											{#if entry.fans_module}
												<Badge variant="default">Yes</Badge>
											{:else}
												<Badge variant="outline">No</Badge>
											{/if}
										</Table.Cell>
										<Table.Cell class="text-center">
											{#if entry.enigoo_involved}
												<Badge variant="default">Yes</Badge>
											{:else}
												<Badge variant="outline">No</Badge>
											{/if}
										</Table.Cell>
										<Table.Cell class="text-sm text-muted-foreground">{entry.note ?? ''}</Table.Cell>
									</Table.Row>
								{/snippet}
							</ContextMenuPrimitive.Trigger>
							<ContextMenu.Content>
								<ContextMenu.Label class="text-xs text-muted-foreground">
									{tenant?.name ?? entry.mongo_id}
								</ContextMenu.Label>
								<ContextMenu.Separator />
								<ContextMenu.Item onclick={() => openEdit(entry)}>Edit</ContextMenu.Item>
								<ContextMenu.Separator />
								<ContextMenu.Item
									class="text-destructive focus:text-destructive"
									onclick={() => openDelete(entry)}
								>
									Delete
								</ContextMenu.Item>
							</ContextMenu.Content>
						</ContextMenu.Root>
					{:else}
						<Table.Row>
							<Table.Cell class="py-8 text-center text-muted-foreground" colspan={12}>
								{search.trim()
									? 'No entries match your search.'
									: 'No onboarding entries yet. Add your first entry.'}
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</div>
	{/if}
</div>

<!-- Shared form snippet -->
{#snippet onboardingForm(form: ReturnType<typeof blankForm>)}
	<!-- Tenant combobox -->
	{@const comboTenants = form.tenantSearch.trim()
		? allTenants.filter((t) => normalize(t.name).includes(normalize(form.tenantSearch)))
		: allTenants}
	<div class="space-y-1.5">
		<Label>Tenant</Label>
		<Combobox.Root
			type="single"
			bind:value={form.mongo_id}
			items={comboTenants.map((t) => ({ value: t.id, label: t.name }))}
			onValueChange={(v) => (form.tenantSearch = tenantMap().get(v)?.name ?? '')}
		>
			<Combobox.Input
				placeholder="Search tenants…"
				defaultValue={form.tenantSearch}
				oninput={(e) => (form.tenantSearch = (e.target as HTMLInputElement).value)}
				class="bg-input/50 focus-visible:border-ring focus-visible:ring-ring/30 h-9 w-full rounded-3xl border border-transparent px-3 py-1 text-sm outline-none transition-[color,box-shadow,background-color] placeholder:text-muted-foreground focus-visible:ring-3"
			/>
			<Combobox.Content
				class="bg-popover text-popover-foreground data-open:animate-in data-closed:animate-out data-closed:fade-out-0 data-open:fade-in-0 data-closed:zoom-out-95 data-open:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=top]:slide-in-from-bottom-2 ring-foreground/5 z-50 max-h-64 min-w-[var(--bits-combobox-anchor-width)] overflow-y-auto rounded-3xl p-1 shadow-lg ring-1 duration-100"
				sideOffset={4}
			>
				{#each comboTenants as tenant (tenant.id)}
					<Combobox.Item
						value={tenant.id}
						label={tenant.name}
						class="focus:bg-accent data-highlighted:bg-accent data-highlighted:text-accent-foreground relative flex w-full cursor-default items-center gap-2.5 rounded-2xl py-2 pl-3 pr-8 text-sm font-medium outline-none select-none"
					>
						{#snippet children({ selected })}
							{#if tenant.club_logo}
								<img
									src={tenant.club_logo}
									alt={tenant.name}
									class="h-6 w-6 shrink-0 rounded-full object-contain"
								/>
							{:else}
								<div
									class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-muted text-xs font-semibold text-muted-foreground"
								>
									{tenant.name.slice(0, 2).toUpperCase()}
								</div>
							{/if}
							<span class="flex-1 truncate">{tenant.name}</span>
							{#if selected}
								<span class="absolute end-2 text-primary">✓</span>
							{/if}
						{/snippet}
					</Combobox.Item>
				{/each}
				{#if comboTenants.length === 0}
					<div class="py-4 text-center text-sm text-muted-foreground">No tenants found.</div>
				{/if}
			</Combobox.Content>
		</Combobox.Root>
	</div>

	<div class="grid grid-cols-2 gap-4">
		<div class="space-y-1.5">
			<Label for="date-training">Training Date</Label>
			<Input id="date-training" type="date" bind:value={form.date_training} />
		</div>
		<div class="space-y-1.5">
			<Label for="price">Price</Label>
			<Input id="price" type="number" min="0" bind:value={form.price} placeholder="0" required />
		</div>
		<div class="space-y-1.5">
			<Label>Currency</Label>
			<Select.Root type="single" bind:value={form.currency}>
				<Select.Trigger class="w-full">
					{CURRENCY_LABELS[form.currency] ?? 'Select'}
				</Select.Trigger>
				<Select.Content>
					{#each Object.values(Currency) as c (c)}
						<Select.Item value={c}>{CURRENCY_LABELS[c]}</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
		</div>
		<div class="space-y-1.5">
			<Label for="invoiced-date">Invoiced Date</Label>
			<Input id="invoiced-date" type="date" bind:value={form.invoiced_date} />
		</div>
	</div>

	<div class="space-y-1.5">
		<Label for="note">Note</Label>
		<textarea
			id="note"
			bind:value={form.note}
			placeholder="Optional note…"
			rows={3}
			class="bg-input/50 focus-visible:border-ring focus-visible:ring-ring/30 w-full rounded-3xl border border-transparent px-3 py-2 text-sm outline-none transition-[color,box-shadow,background-color] placeholder:text-muted-foreground focus-visible:ring-3 resize-none"
		></textarea>
	</div>

	<div class="grid grid-cols-2 gap-3 pt-1">
		<label class="flex cursor-pointer items-center gap-2.5">
			<Checkbox bind:checked={form.paid} id="paid" />
			<Label for="paid" class="cursor-pointer font-normal">Paid</Label>
		</label>
		<label class="flex cursor-pointer items-center gap-2.5">
			<Checkbox bind:checked={form.invoiced} id="invoiced" />
			<Label for="invoiced" class="cursor-pointer font-normal">Invoiced</Label>
		</label>
		<label class="flex cursor-pointer items-center gap-2.5">
			<Checkbox bind:checked={form.business_module} id="business-module" />
			<Label for="business-module" class="cursor-pointer font-normal">Business Module</Label>
		</label>
		<label class="flex cursor-pointer items-center gap-2.5">
			<Checkbox bind:checked={form.fans_module} id="fans-module" />
			<Label for="fans-module" class="cursor-pointer font-normal">Fans Module</Label>
		</label>
		<label class="flex cursor-pointer items-center gap-2.5">
			<Checkbox bind:checked={form.enigoo_involved} id="enigoo-involved" />
			<Label for="enigoo-involved" class="cursor-pointer font-normal">Enigoo Involved</Label>
		</label>
	</div>
{/snippet}

<!-- Add Entry Dialog -->
<Dialog.Root bind:open={addOpen}>
	<Dialog.Content class="sm:max-w-[520px]">
		<Dialog.Header>
			<Dialog.Title>Add Onboarding Entry</Dialog.Title>
			<Dialog.Description>Record a new onboarding or training session for a tenant.</Dialog.Description>
		</Dialog.Header>
		<form class="space-y-4 py-2" onsubmit={submitAdd}>
			{@render onboardingForm(addForm)}
			{#if addError}
				<p class="text-sm text-destructive">{addError}</p>
			{/if}
			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (addOpen = false)}>Cancel</Button>
				<Button type="submit" disabled={addMutation.isPending}>
					{addMutation.isPending ? 'Creating...' : 'Create'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Edit Entry Dialog -->
<Dialog.Root bind:open={editOpen}>
	<Dialog.Content class="sm:max-w-[520px]">
		<Dialog.Header>
			<Dialog.Title>Edit Onboarding Entry</Dialog.Title>
			<Dialog.Description>
				Update the onboarding entry for <span class="font-medium"
					>{tenantMap().get(editingEntry?.mongo_id ?? '')?.name ??
						editingEntry?.mongo_id ??
						''}</span
				>.
			</Dialog.Description>
		</Dialog.Header>
		<form class="space-y-4 py-2" onsubmit={submitEdit}>
			{@render onboardingForm(editForm)}
			{#if editError}
				<p class="text-sm text-destructive">{editError}</p>
			{/if}
			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (editOpen = false)}>Cancel</Button>
				<Button type="submit" disabled={editMutation.isPending}>
					{editMutation.isPending ? 'Saving...' : 'Save'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Delete Confirm Dialog -->
<Dialog.Root bind:open={deleteOpen}>
	<Dialog.Content class="sm:max-w-[400px]">
		<Dialog.Header>
			<Dialog.Title>Delete Onboarding Entry</Dialog.Title>
			<Dialog.Description>
				Are you sure you want to delete the entry for
				<strong
					>{tenantMap().get(deletingEntry?.mongo_id ?? '')?.name ??
						deletingEntry?.mongo_id ??
						''}</strong
				>? This cannot be undone.
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (deleteOpen = false)}>Cancel</Button>
			<Button
				variant="destructive"
				disabled={deleteMutation.isPending}
				onclick={() => {
					if (deletingEntry) deleteMutation.mutate({ id: deletingEntry.id });
				}}
			>
				{deleteMutation.isPending ? 'Deleting...' : 'Delete'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
