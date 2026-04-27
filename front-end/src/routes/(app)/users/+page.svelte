<script lang="ts">
	import UsersTable from './components/UsersTable.svelte';
	import AddUserDialog from './components/AddUserDialog.svelte';
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-semibold tracking-tight">Users</h1>
			<p class="text-sm text-muted-foreground">Manage user accounts and roles.</p>
		</div>
		<AddUserDialog />
	</div>

	<UsersTable />
</div>
