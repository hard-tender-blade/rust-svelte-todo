<script lang="ts">
	import { createListAllInvoicingEntries } from '$lib/api/generated/admin-invoicing/admin-invoicing';
	import { createListAllOnboardingEntries } from '$lib/api/generated/admin-onboarding/admin-onboarding';
	import { ChartContainer, type ChartConfig } from '$lib/components/ui/chart/index.js';
	import { BarChart, Tooltip } from 'layerchart';

	const invoicingQuery = createListAllInvoicingEntries();
	const onboardingQuery = createListAllOnboardingEntries();

	const invoicingEntries = $derived(
		invoicingQuery.data?.status === 200 ? invoicingQuery.data.data : []
	);
	const onboardingEntries = $derived(
		onboardingQuery.data?.status === 200 ? onboardingQuery.data.data : []
	);

	function toMonthKey(s: string | null | undefined): string | null {
		return s ? s.slice(0, 7) : null;
	}

	function keyToLabel(k: string): string {
		const [y, m] = k.split('-').map(Number);
		return new Date(y, m - 1, 1).toLocaleDateString('en-US', { month: 'short', year: '2-digit' });
	}

	function fmtNum(n: number): string {
		return n.toLocaleString('cs-CZ');
	}

	type MonthData = {
		label: string;
		monthKey: string;
		invoices: number;
		onboarding: number;
	};

	const chartData = $derived.by((): MonthData[] => {
		const inv = new Map<string, number>();
		for (const e of invoicingEntries) {
			const k = toMonthKey(e.date);
			if (k) inv.set(k, (inv.get(k) ?? 0) + e.price);
		}

		const onb = new Map<string, number>();
		for (const e of onboardingEntries) {
			const k = toMonthKey(e.invoiced_date);
			if (k) onb.set(k, (onb.get(k) ?? 0) + e.price);
		}

		const allKeys = new Set([...inv.keys(), ...onb.keys()]);

		return [...allKeys]
			.sort()
			.map((k) => ({
				label: keyToLabel(k),
				monthKey: k,
				invoices: inv.get(k) ?? 0,
				onboarding: onb.get(k) ?? 0
			}));
	});

	const chartConfig: ChartConfig = {
		invoices:   { label: 'Invoices',   color: 'var(--chart-2)' },
		onboarding: { label: 'Onboarding', color: 'var(--chart-3)' }
	};

	const isLoading = $derived(invoicingQuery.isPending || onboardingQuery.isPending);
	const isError = $derived(invoicingQuery.isError || onboardingQuery.isError);
</script>

<div class="space-y-6">
	<div>
		<h1 class="text-2xl font-semibold tracking-tight">Overview</h1>
		<p class="text-sm text-muted-foreground">Invoicing and onboarding revenue by month.</p>
	</div>

	<div class="w-full rounded-lg border bg-card px-5 pb-5 pt-4">
		<div class="mb-3 flex items-center gap-5">
			<div class="flex items-center gap-1.5">
				<span class="inline-block h-2.5 w-2.5 rounded-[2px]" style="background: var(--chart-2)"></span>
				<span class="text-xs text-muted-foreground">Invoices</span>
			</div>
			<div class="flex items-center gap-1.5">
				<span class="inline-block h-2.5 w-2.5 rounded-[2px]" style="background: var(--chart-3)"></span>
				<span class="text-xs text-muted-foreground">Onboarding</span>
			</div>
		</div>

		{#if isLoading}
			<div class="flex h-72 items-center justify-center text-sm text-muted-foreground">Loading…</div>
		{:else if isError}
			<div class="flex h-72 items-center justify-center text-sm text-destructive">Failed to load data.</div>
		{:else if chartData.length === 0}
			<div class="flex h-72 items-center justify-center text-sm text-muted-foreground">No data yet.</div>
		{:else}
			<ChartContainer config={chartConfig} class="h-72 w-full [&>*]:flex-1 [&>*]:min-w-0">
				<BarChart
					data={chartData}
					x={(d) => d.label}
					y={(d) => d.invoices + d.onboarding}
					series={[
						{ key: 'invoices',   label: 'Invoices',   color: 'var(--chart-2)' },
						{ key: 'onboarding', label: 'Onboarding', color: 'var(--chart-3)' }
					]}
					seriesLayout="stack"
					bandPadding={0.35}
					legend={false}
					rule={false}
				>
					{#snippet tooltip({ context })}
						{@const d = context.tooltip.data as MonthData | null}
						{#if d}
							<Tooltip.Root variant="none">
								<div class="border-border/50 bg-background grid min-w-[10rem] gap-1.5 rounded-lg border px-2.5 py-1.5 text-xs shadow-xl">
									<p class="font-medium">{d.label}</p>
									<div class="grid gap-1">
										<div class="flex items-center justify-between gap-6">
											<div class="flex items-center gap-1.5">
												<span class="inline-block h-2.5 w-2.5 rounded-[2px]" style="background: var(--chart-2)"></span>
												<span class="text-muted-foreground">Invoices</span>
											</div>
											<span class="font-mono font-medium tabular-nums">{fmtNum(d.invoices)}</span>
										</div>
										<div class="flex items-center justify-between gap-6">
											<div class="flex items-center gap-1.5">
												<span class="inline-block h-2.5 w-2.5 rounded-[2px]" style="background: var(--chart-3)"></span>
												<span class="text-muted-foreground">Onboarding</span>
											</div>
											<span class="font-mono font-medium tabular-nums">{fmtNum(d.onboarding)}</span>
										</div>
										<div class="mt-0.5 flex items-center justify-between gap-6 border-t pt-1">
											<span class="text-muted-foreground">Total</span>
											<span class="font-mono font-medium tabular-nums">{fmtNum(d.invoices + d.onboarding)}</span>
										</div>
									</div>
								</div>
							</Tooltip.Root>
						{/if}
					{/snippet}
				</BarChart>
			</ChartContainer>
		{/if}
	</div>
</div>
