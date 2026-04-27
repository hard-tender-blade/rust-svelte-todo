<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { useQueryClient } from '@tanstack/svelte-query';
	import { createUpdateUser, getListUsersQueryKey } from '$lib/api/generated/users/users';
	import { UserRole, type User } from '$lib/api/generated/rustSvelteTodo.schemas';
	import { createForm } from 'felte';
	import { validator } from '@felte/validator-yup';
	import * as yup from 'yup';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Field from '$lib/components/ui/field/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { ROLE_LABELS } from '../users.utils';

	let { user, onclose }: { user: User | null; onclose: () => void } = $props();

	const queryClient = useQueryClient();
	const mutation = createUpdateUser();

	const schema = yup.object({
		full_name: yup.string().trim().required('Name is required'),
		role: yup.string().oneOf(Object.values(UserRole)).required('Role is required')
	});

	let submitAttempted = $state(false);
	const fe = (errs: string[] | null | undefined) =>
		submitAttempted ? (errs ?? []).map((message) => ({ message })) : [];

	const { form, data, errors, isSubmitting, reset, setFields } = createForm({
		initialValues: { full_name: '', role: UserRole.viewer as string },
		extend: validator({ schema }),
		onSubmit: async (values) => {
			await mutation.mutateAsync({
				id: user!.id,
				data: {
					full_name: values.full_name,
					role: values.role as (typeof UserRole)[keyof typeof UserRole]
				}
			});
			queryClient.invalidateQueries({ queryKey: getListUsersQueryKey() });
			toast.success('User updated');
			handleClose();
		},
		onError: (err: unknown) => {
			console.error(err);
			toast.error('Failed to update user');
		}
	});

	function handleClose() {
		if (!open) return;
		submitAttempted = false;
		reset();
		onclose();
	}

	$effect(() => {
		if (user) {
			reset();
			setFields({ full_name: user.full_name, role: user.role });
		}
	});

	const open = $derived(user !== null);
</script>

<Dialog.Root
	{open}
	onOpenChange={(v) => {
		if (!v) handleClose();
	}}
>
	<Dialog.Content class="sm:max-w-110" showCloseButton={false}>
		<Dialog.Header>
			<Dialog.Title>Edit User</Dialog.Title>
			<Dialog.Description>Update name and role for {user?.full_name ?? ''}.</Dialog.Description>
		</Dialog.Header>

		<form use:form onsubmit={() => (submitAttempted = true)}>
			<Field.Group>
				<Field.Field>
					<Field.Label for="edit-name">Full Name</Field.Label>
					<Input id="edit-name" name="full_name" />
					<Field.Error errors={fe($errors.full_name)} />
				</Field.Field>

				<Field.Field>
					<Field.Label>Role</Field.Label>
					<Select.Root
						type="single"
						value={$data.role}
						onValueChange={(v) => setFields('role', v, true)}
					>
						<Select.Trigger class="w-full">
							{ROLE_LABELS[$data.role] ?? 'Select role'}
						</Select.Trigger>
						<Select.Content>
							{#each Object.values(UserRole) as role (role)}
								<Select.Item value={role}>{ROLE_LABELS[role]}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
					<Field.Error errors={fe($errors.role)} />
				</Field.Field>
			</Field.Group>

			<Dialog.Footer class="mt-6">
				<Button type="button" variant="outline" onclick={handleClose}>Cancel</Button>
				<Button type="submit" disabled={$isSubmitting}>
					{$isSubmitting ? 'Saving...' : 'Save'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
