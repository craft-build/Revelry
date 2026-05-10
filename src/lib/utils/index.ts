import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

/**
 * Merge Tailwind class names with conflict resolution.
 * Used everywhere Bits.ui components accept a `class` prop.
 */
export function cn(...inputs: ClassValue[]): string {
	return twMerge(clsx(inputs));
}

/**
 * Format an ISO timestamp for the conversation sidebar.
 * Today → time only, this week → weekday, older → date.
 */
export function formatRelativeTime(iso: string): string {
	const date = new Date(iso);
	const now = new Date();
	const diffMs = now.getTime() - date.getTime();
	const diffDays = diffMs / 86_400_000;

	if (diffDays < 1 && date.getDate() === now.getDate()) {
		return date.toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' });
	}
	if (diffDays < 7) {
		return date.toLocaleDateString([], { weekday: 'short' });
	}
	return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
}

/**
 * Approximate token count using a 4-char-per-token heuristic.
 * The real app will swap this for tiktoken-rs via the backend bridge.
 */
export function estimateTokens(text: string): number {
	if (!text) return 0;
	return Math.ceil(text.length / 4);
}

/**
 * Generate a stable initial-letter seed for AdwAvatar-style fallbacks.
 */
export function avatarInitials(name: string): string {
	const parts = name.trim().split(/\s+/).slice(0, 2);
	return parts
		.map((p) => p[0]?.toUpperCase() ?? '')
		.join('')
		.slice(0, 2);
}
