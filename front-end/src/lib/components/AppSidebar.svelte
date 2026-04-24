<script lang="ts">
	import { createMe, createSignout } from '$lib/api/generated/auth/auth';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import * as Avatar from '$lib/components/ui/avatar/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { Users, Building2, LogOut, ChevronsUpDown, ChevronRight, TrendingUp, FileText, Wallet, TableOfContents, GraduationCap, LayoutDashboard } from '@lucide/svelte';
	import * as Collapsible from '$lib/components/ui/collapsible/index.js';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';

	const meQuery = createMe();
	const user = $derived(meQuery.data?.status === 200 ? meQuery.data.data : null);

	const signoutMutation = createSignout(() => ({
		mutation: {
			onSuccess: () => goto('/auth/signin')
		}
	}));

	function handleSignOut() {
		signoutMutation.mutate();
	}

	function getInitials(name: string) {
		return name
			.split(' ')
			.slice(0, 2)
			.map((n) => n[0])
			.join('')
			.toUpperCase();
	}

	const navItems = [
		{ href: '/tenants', label: 'Tenants', icon: Building2 },
		{ href: '/users', label: 'Users', icon: Users }
	];

	const financesItems = [
		{ href: '/finances/overview', label: 'Overview', icon: LayoutDashboard },
		{ href: '/finances/invoices', label: 'Invoices', icon: TableOfContents },
		{ href: '/finances/plan', label: 'Plan', icon: TrendingUp },
		{ href: '/finances/billing', label: 'Billing', icon: FileText },
		{ href: '/finances/onboarding', label: 'Onboarding', icon: GraduationCap }
	];
</script>

<Sidebar.Root collapsible="icon">
	<!-- Logo -->
	<Sidebar.Header>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton size="lg" tooltipContent="Relatoo">
					{#snippet child({ props })}
						<a href="/" {...props}>
							<!--
								The logo SVG has a green icon mark + white wordmark text.
								- Expanded: show full logo on a dark pill so white text is visible.
								- Collapsed (icon mode): show just the green R icon paths (no dark bg needed).
							-->

							<!-- Expanded: full wordmark on dark background -->
							<div
								class="flex h-8 shrink-0 items-center overflow-hidden rounded-lg px-2.5 group-data-[collapsible=icon]:hidden"
							>
								<img src="/logo.png" alt="Relatoo" class="h-[18px] w-auto" />
							</div>

							<!-- Collapsed: small icon -->
							<img
								src="/logo-small.png"
								alt="Relatoo"
								class="mx-auto hidden size-4 object-contain group-data-[collapsible=icon]:block"
							/>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Header>

	<!-- Nav -->
	<Sidebar.Content>
		<Sidebar.Group>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					{#each navItems as item}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton
								isActive={$page.url.pathname.startsWith(item.href)}
								tooltipContent={item.label}
							>
								{#snippet child({ props })}
									<a href={item.href} {...props}>
										<item.icon />
										<span>{item.label}</span>
									</a>
								{/snippet}
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
					{/each}

					<Collapsible.Root
						open={$page.url.pathname.startsWith('/finances')}
						class="group/collapsible"
					>
						{#snippet child({ props })}
							<Sidebar.MenuItem {...props}>
								<Collapsible.Trigger>
									{#snippet child({ props })}
										<Sidebar.MenuButton {...props} tooltipContent="Finances">
											<Wallet />
											<span>Finances</span>
											<ChevronRight
												class="ms-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90"
											/>
										</Sidebar.MenuButton>
									{/snippet}
								</Collapsible.Trigger>
								<Collapsible.Content>
									<Sidebar.MenuSub>
										{#each financesItems as subItem}
											<Sidebar.MenuSubItem>
												<Sidebar.MenuSubButton
													isActive={$page.url.pathname.startsWith(subItem.href)}
												>
													{#snippet child({ props })}
														<a href={subItem.href} {...props}>
															<subItem.icon />
															<span>{subItem.label}</span>
														</a>
													{/snippet}
												</Sidebar.MenuSubButton>
											</Sidebar.MenuSubItem>
										{/each}
									</Sidebar.MenuSub>
								</Collapsible.Content>
							</Sidebar.MenuItem>
						{/snippet}
					</Collapsible.Root>
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
	</Sidebar.Content>

	<!-- User -->
	<Sidebar.Footer>
		{#if meQuery.isLoading}
			<Sidebar.Menu>
				<Sidebar.MenuItem>
					<Sidebar.MenuButton size="lg">
						<div class="h-8 w-8 animate-pulse rounded-lg bg-muted"></div>
						<div class="grid flex-1 gap-1">
							<div class="h-3 w-24 animate-pulse rounded bg-muted"></div>
							<div class="h-2 w-32 animate-pulse rounded bg-muted"></div>
						</div>
					</Sidebar.MenuButton>
				</Sidebar.MenuItem>
			</Sidebar.Menu>
		{:else if user}
			<Sidebar.Menu>
				<Sidebar.MenuItem>
					<DropdownMenu.Root>
						<DropdownMenu.Trigger>
							{#snippet child({ props })}
								<Sidebar.MenuButton
									size="lg"
									class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
									{...props}
								>
									<Avatar.Root class="h-8 w-8 rounded-lg">
										<Avatar.Fallback
											class="rounded-lg bg-primary/10 text-xs font-semibold text-primary"
										>
											{getInitials(user.full_name)}
										</Avatar.Fallback>
									</Avatar.Root>
									<div class="grid flex-1 text-left text-sm leading-tight">
										<span class="truncate font-medium">{user.full_name}</span>
										<span class="truncate text-xs text-muted-foreground">{user.email}</span>
									</div>
									<ChevronsUpDown class="ml-auto size-4" />
								</Sidebar.MenuButton>
							{/snippet}
						</DropdownMenu.Trigger>
						<DropdownMenu.Content class="w-56" side="right" align="end" sideOffset={4}>
							<DropdownMenu.Label class="font-normal">
								<div class="flex flex-col gap-0.5">
									<span class="text-sm font-medium">{user.full_name}</span>
									<span class="truncate text-xs text-muted-foreground">{user.email}</span>
								</div>
							</DropdownMenu.Label>
							<DropdownMenu.Separator />
							<DropdownMenu.Item
								onclick={handleSignOut}
								class="text-destructive focus:text-destructive"
							>
								<LogOut class="mr-2 h-4 w-4" />
								Sign out
							</DropdownMenu.Item>
						</DropdownMenu.Content>
					</DropdownMenu.Root>
				</Sidebar.MenuItem>
			</Sidebar.Menu>
		{/if}
	</Sidebar.Footer>
</Sidebar.Root>
