/**
 * Decides whether typed input may be displayed on the on-screen keyboard.
 *
 * Two signals arrive from the backend with very different lifetimes:
 *
 *   canonical_secret — canonical mode with echo off (getpass, `read -s`). True
 *   for exactly as long as the read lasts, so it gates display directly.
 *
 *   program_secret — a secret-reading program owns the terminal in raw mode,
 *   which is sudo with pwfeedback. This cannot say when the read *ends*: after
 *   `sudo su` authenticates, sudo remains the foreground process group leader
 *   for the whole root session. Treating it as a live truth left the keyboard
 *   dark until the user exited root. It is therefore latched on its rising
 *   edge and released when the read actually finishes — on Enter.
 */
export interface SecureInputSample {
  canonical_secret: boolean;
  program_secret: boolean;
  at_shell_prompt: boolean;
}

const IDLE: SecureInputSample = {
  canonical_secret: false,
  program_secret: false,
  at_shell_prompt: false,
};

export class SecureInputTracker {
  /** True while typed input must not be displayed. */
  secure = false;

  private latched = false;
  /**
   * Starts false so a reader already holding the terminal when the panel opens
   * counts as a rising edge. Deliberate: opening the panel during a password
   * prompt is indistinguishable from opening it during a root session, and the
   * safe reading is "a secret may be in progress". The cost is one Enter press
   * to re-enable highlighting; the cost of the other choice is a shown password.
   */
  private hadProgramSecret = false;
  private last: SecureInputSample = IDLE;

  /** A poll result from the backend. */
  sample(s: SecureInputSample): boolean {
    if (s.at_shell_prompt) this.latched = false;
    if (s.program_secret && !this.hadProgramSecret) this.latched = true;
    this.hadProgramSecret = s.program_secret;
    this.last = s;
    return this.recompute();
  }

  /** A password-looking prompt appeared in the terminal output. */
  promptSeen(): boolean {
    this.latched = true;
    return this.recompute();
  }

  /** The line was submitted, so any latched read has ended. */
  submitted(): boolean {
    this.latched = false;
    return this.recompute();
  }

  /** Backend unreachable — assume a secret is being read. */
  unknown(): boolean {
    this.secure = true;
    return this.secure;
  }

  private recompute(): boolean {
    this.secure = this.last.canonical_secret || this.latched;
    return this.secure;
  }
}
