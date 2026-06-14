import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { convertFileSrc } from "@tauri-apps/api/core";

// ── Steering message parsing ──────────────────────────────────────────────────

export interface ParsedInput {
  /** In-character text (empty string if steer-only). */
  content: string;
  /** Steering instruction text (empty string if none). */
  steering: string;
}

/**
 * Parse `/steer` commands from user input.
 * Detects lines starting with `/steer ` (case-insensitive, space required).
 * Extracts all `/steer` lines as a single joined steering string.
 * Remaining non-`/steer` lines become content.
 */
export function parseSteering(input: string): ParsedInput {
  const lines = input.split("\n");
  const steerLines: string[] = [];
  const contentLines: string[] = [];

  for (const line of lines) {
    if (/^\/steer\s/i.test(line)) {
      steerLines.push(line.replace(/^\/steer\s+/i, ""));
    } else {
      contentLines.push(line);
    }
  }

  return {
    content: contentLines.join("\n").trim(),
    steering: steerLines.join("\n").trim(),
  };
}

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
    return date.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
  }
  if (diffDays < 7) {
    return date.toLocaleDateString([], { weekday: "short" });
  }
  return date.toLocaleDateString([], { month: "short", day: "numeric" });
}

/**
 * Client-side UI estimate using a 4-char-per-token heuristic.
 * Used only for the input context bar before a round-trip; accurate
 * counts come from the backend (tiktoken-rs) via ContextBudget.
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
    .map((p) => p[0]?.toUpperCase() ?? "")
    .join("")
    .slice(0, 2);
}

/**
 * Convert an absolute avatar_path (from the Rust backend) into a displayable URL
 * using Tauri's `convertFileSrc()` which produces `asset://` or `http://asset.localhost/` URLs.
 * Returns `undefined` if the path is empty or conversion fails, so the Avatar
 * component falls back to initials.
 */
export function resolveAvatarUrl(avatarPath?: string): string | undefined {
  if (!avatarPath) return undefined;
  try {
    return convertFileSrc(avatarPath);
  } catch {
    return undefined;
  }
}
