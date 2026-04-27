import { UserRole } from '$lib/api/generated/rustSvelteTodo.schemas';

export const ROLE_LABELS: Record<string, string> = {
	[UserRole.admin]: 'Admin',
	[UserRole.editor]: 'Editor',
	[UserRole.viewer]: 'Viewer'
};

export function roleBadgeVariant(role: string): 'default' | 'secondary' | 'outline' {
	if (role === UserRole.admin) return 'default';
	if (role === UserRole.editor) return 'secondary';
	return 'outline';
}
