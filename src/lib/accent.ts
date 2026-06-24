import type { AccentColor } from "@/types/domain";

/**
 * Category colours are data-driven, so we can't use static Tailwind classes
 * (they'd be purged). Instead we read the CSS custom properties defined in
 * index.css @theme. One accent per category — the core CosyPOS device.
 */
export const accentVar = (c: AccentColor): string => `var(--color-${c})`;

/** A faint tint of the accent for selected rows / button backgrounds. */
export const accentTint = (c: AccentColor, alpha = 0.16): string =>
  `color-mix(in srgb, var(--color-${c}) ${alpha * 100}%, transparent)`;

export const ACCENT_CHOICES: AccentColor[] = [
  "blue",
  "pink",
  "purple",
  "mint",
  "amber",
  "coral",
];
