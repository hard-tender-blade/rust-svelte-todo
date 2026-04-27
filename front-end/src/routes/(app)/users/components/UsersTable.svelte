<script lang="ts">
	import { createListUsers } from '$lib/api/generated/users/users';
	import type { User } from '$lib/api/generated/rustSvelteTodo.schemas';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';
	import { ContextMenu as ContextMenuPrimitive } from 'bits-ui';
	import { ROLE_LABELS, roleBadgeVariant } from '../users.utils';
	import EditUserDialog from './EditUserDialog.svelte';
	import DeleteUserDialog from './DeleteUserDialog.svelte';

	const usersQuery = createListUsers();
	const users = $derived(usersQuery.data?.status === 200 ? usersQuery.data.data : []);

	let editingUser = $state<User | null>(null);
	let deletingUser = $state<User | null>(null);
</script>

{#if usersQuery.isPending}
	<div class="text-sm text-muted-foreground">Loading...</div>
{:else if usersQuery.isError}
	<div class="text-sm text-destructive">Failed to load users.</div>
{:else}
	<div class="rounded-lg border">
		<Table.Root>
			<Table.Header>
				<Table.Row>
					<Table.Head>Name</Table.Head>
					<Table.Head>Email</Table.Head>
					<Table.Head>Role</Table.Head>
				</Table.Row>
			</Table.Header>
			<Table.Body>
				{#each users as user (user.id)}
					<ContextMenu.Root>
						<ContextMenuPrimitive.Trigger>
							{#snippet child({ props })}
								<Table.Row {...props} class="cursor-context-menu select-none">
									<Table.Cell class="font-medium">{user.full_name}</Table.Cell>
									<Table.Cell class="text-muted-foreground">{user.email}</Table.Cell>
									<Table.Cell>
										<Badge variant={roleBadgeVariant(user.role)}>
											{ROLE_LABELS[user.role] ?? user.role}
										</Badge>
									</Table.Cell>
								</Table.Row>
							{/snippet}
						</ContextMenuPrimitive.Trigger>
						<ContextMenu.Content>
							<ContextMenu.Label class="text-xs text-muted-foreground">
								{user.full_name}
							</ContextMenu.Label>
							<ContextMenu.Separator />
							<ContextMenu.Item onclick={() => (editingUser = user)}>Edit</ContextMenu.Item>
							<ContextMenu.Separator />
							<ContextMenu.Item
								class="text-destructive focus:text-destructive"
								onclick={() => (deletingUser = user)}
							>
								Delete
							</ContextMenu.Item>
						</ContextMenu.Content>
					</ContextMenu.Root>
				{:else}
					<Table.Row>
						<Table.Cell class="py-8 text-center text-muted-foreground" colspan={3}>
							No users found.
						</Table.Cell>
					</Table.Row>
				{/each}
			</Table.Body>
		</Table.Root>
	</div>
{/if}

<EditUserDialog user={editingUser} onclose={() => (editingUser = null)} />
<DeleteUserDialog user={deletingUser} onclose={() => (deletingUser = null)} />
