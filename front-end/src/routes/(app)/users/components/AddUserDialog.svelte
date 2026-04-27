<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { useQueryClient } from '@tanstack/svelte-query';
	import { createCreateUser, getListUsersQueryKey } from '$lib/api/generated/users/users';
	import { UserRole } from '$lib/api/generated/rustSvelteTodo.schemas';
	import { createForm } from 'felte';
	import { validator } from '@felte/validator-yup';
	import * as yup from 'yup';
	import { buttonVariants } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Field from '$lib/components/ui/field/index.js';
	import { Eye, EyeOff, Wand2 } from '@lucide/svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { ROLE_LABELS } from '../users.utils';

	const queryClient = useQueryClient();
	const mutation = createCreateUser();

	const schema = yup.object({
		full_name: yup.string().trim().required('Name is required'),
		email: yup.string().email('Invalid email').required('Email is required'),
		password: yup.string().min(8, 'Minimum 8 characters').required('Password is required'),
		role: yup.string().oneOf(Object.values(UserRole)).required('Role is required')
	});

	let open = $state(false);
	let submitAttempted = $state(false);
	let showPassword = $state(false);

	const fe = (errs: string[] | null | undefined) =>
		submitAttempted ? (errs ?? []).map((message) => ({ message })) : [];

	const { form, data, errors, isSubmitting, reset, setFields } = createForm({
		initialValues: { full_name: '', email: '', password: '', role: UserRole.viewer as string },
		extend: validator({ schema }),
		onSubmit: async (values) => {
			await mutation.mutateAsync({
				data: {
					full_name: values.full_name,
					email: values.email,
					password: values.password,
					role: values.role as (typeof UserRole)[keyof typeof UserRole]
				}
			});
			queryClient.invalidateQueries({ queryKey: getListUsersQueryKey() });
			toast.success('User created');
			open = false;
		},
		onError: (err: unknown) => {
			console.error(err);
			toast.error('Failed to create user');
		}
	});

	$effect(() => {
		if (!open) {
			submitAttempted = false;
			showPassword = false;
			reset();
		}
	});

	function generatePassword() {
		const upper = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
		const lower = 'abcdefghijklmnopqrstuvwxyz';
		const digits = '0123456789';
		const symbols = '!@#$%^&*()-_=+[]{}';
		const all = upper + lower + digits + symbols;
		const rand = (s: string) => s[Math.floor(Math.random() * s.length)];
		const rest = Array.from({ length: 12 }, () => rand(all));
		const pw = [rand(upper), rand(lower), rand(digits), rand(symbols), ...rest];
		for (let i = pw.length - 1; i > 0; i--) {
			const j = Math.floor(Math.random() * (i + 1));
			[pw[i], pw[j]] = [pw[j], pw[i]];
		}
		setFields('password', pw.join(''), true);
		showPassword = true;
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Trigger type="button" class={buttonVariants()}>+ Add User</Dialog.Trigger>

	<Dialog.Content class="sm:max-w-110" showCloseButton={false}>
		<Dialog.Header>
			<Dialog.Title>Add User</Dialog.Title>
			<Dialog.Description>Create a new user account.</Dialog.Description>
		</Dialog.Header>

		<form use:form onsubmit={() => (submitAttempted = true)}>
			<Field.Group>
				<Field.Field>
					<Field.Label for="add-name">Full Name</Field.Label>
					<Input id="add-name" name="full_name" placeholder="Jane Doe" />
					<Field.Error errors={fe($errors.full_name)} />
				</Field.Field>

				<Field.Field>
					<Field.Label for="add-email">Email</Field.Label>
					<Input id="add-email" name="email" type="email" placeholder="jane@example.com" />
					<Field.Error errors={fe($errors.email)} />
				</Field.Field>

				<Field.Field>
					<div class="flex items-center justify-between">
						<Field.Label for="add-password">Password</Field.Label>
						<button
							type="button"
							onclick={generatePassword}
							class="flex items-center gap-1 text-xs text-muted-foreground transition-colors hover:text-foreground"
						>
							<Wand2 class="size-3" />
							Generate
						</button>
					</div>
					<div class="relative">
						<Input
							id="add-password"
							name="password"
							type={showPassword ? 'text' : 'password'}
							value={$data.password}
							placeholder="••••••••"
							class="pr-9 font-mono"
						/>
						<button
							type="button"
							onclick={() => (showPassword = !showPassword)}
							class="absolute inset-y-0 right-0 flex items-center px-2.5 text-muted-foreground transition-colors hover:text-foreground"
							tabindex={-1}
						>
							{#if showPassword}
								<EyeOff class="h-4 w-4" />
							{:else}
								<Eye class="h-4 w-4" />
							{/if}
						</button>
					</div>
					<Field.Error errors={fe($errors.password)} />
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
				<Dialog.Close type="button" class={buttonVariants({ variant: 'outline' })}>
					Cancel
				</Dialog.Close>
				<button type="submit" class={buttonVariants()} disabled={$isSubmitting}>
					{$isSubmitting ? 'Creating...' : 'Create'}
				</button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
