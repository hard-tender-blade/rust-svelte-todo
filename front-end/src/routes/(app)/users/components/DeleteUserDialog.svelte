<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { useQueryClient } from '@tanstack/svelte-query';
	import { createDeleteUser, getListUsersQueryKey } from '$lib/api/generated/users/users';
	import type { User } from '$lib/api/generated/rustSvelteTodo.schemas';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';

	let { user, onclose }: { user: User | null; onclose: () => void } = $props();

	const queryClient = useQueryClient();
	const mutation = createDeleteUser();

	const open = $derived(user !== null);
</script>

<Dialog.Root
	{open}
	onOpenChange={(v) => {
		if (!v) onclose();
	}}
>
	<Dialog.Content class="sm:max-w-110" showCloseButton={false}>
		<Dialog.Header>
			<Dialog.Title>Delete User</Dialog.Title>
			<Dialog.Description>
				Are you sure you want to delete <strong>{user?.full_name}</strong>? This cannot be undone.
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button type="button" variant="outline" onclick={onclose}>Cancel</Button>
			<Button
				type="button"
				variant="destructive"
				disabled={mutation.isPending}
				onclick={async () => {
					if (!user) return;
					try {
						await mutation.mutateAsync({ id: user.id });
						queryClient.invalidateQueries({ queryKey: getListUsersQueryKey() });
						toast.success('User deleted');
						onclose();
					} catch {
						toast.error('Failed to delete user');
					}
				}}
			>
				{mutation.isPending ? 'Deleting...' : 'Delete'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
