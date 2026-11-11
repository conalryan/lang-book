/** Feedback for a single guess. */
export type Hint = "low" | "high" | "correct";

export interface GameConfig {
  readonly min: number;
  readonly max: number;
  readonly maxAttempts: number;
}

export const DEFAULT_CONFIG: GameConfig = { min: 1, max: 100, maxAttempts: 10 };

/** What the player typed, after parsing. */
export type PlayerInput =
  | { readonly kind: "guess"; readonly value: number }
  | { readonly kind: "quit" }
  | { readonly kind: "invalid" };

export type GameOutcome =
  | { readonly kind: "won"; readonly attempts: number }
  | { readonly kind: "lost"; readonly secret: number }
  | { readonly kind: "quit" };
