import { test } from 'node:test';
import assert from 'node:assert/strict';
import { SecureInputTracker, type SecureInputSample } from './secureInput.ts';

const prompt: SecureInputSample = { canonical_secret: false, program_secret: false, at_shell_prompt: true };
const canonicalRead: SecureInputSample = { canonical_secret: true, program_secret: false, at_shell_prompt: true };
const sudoRaw: SecureInputSample = { canonical_secret: false, program_secret: true, at_shell_prompt: false };
const rootShell: SecureInputSample = { canonical_secret: false, program_secret: true, at_shell_prompt: false };

test('an ordinary shell prompt is displayable', () => {
  const t = new SecureInputTracker();
  assert.equal(t.sample(prompt), false);
});

test('a canonical read gates display for exactly its duration', () => {
  const t = new SecureInputTracker();
  t.sample(prompt);
  assert.equal(t.sample(canonicalRead), true);
  assert.equal(t.sample(prompt), false, 'released once the read ends');
});

test('Enter does not release a canonical read that is still running', () => {
  const t = new SecureInputTracker();
  t.sample(canonicalRead);
  assert.equal(t.submitted(), true, 'termios still says a read is in progress');
});

test('sudo su: suppressed while typing, restored after Enter, stays restored', () => {
  const t = new SecureInputTracker();
  t.sample(prompt);

  // sudo takes the terminal and prints its prompt
  assert.equal(t.sample(sudoRaw), true, 'rising edge latches');
  assert.equal(t.promptSeen(), true);

  // typing the password changes nothing
  assert.equal(t.sample(sudoRaw), true);

  // Enter ends the read
  assert.equal(t.submitted(), false, 'highlighting resumes immediately');

  // the root session keeps sudo as process group leader — must stay released.
  // This is the regression: it previously stayed dark until the user exited root.
  for (let i = 0; i < 20; i++) {
    assert.equal(t.sample(rootShell), false, `still displayable at poll ${i}`);
  }
});

test('a rejected password re-latches on the retry prompt', () => {
  const t = new SecureInputTracker();
  t.sample(prompt);
  t.sample(sudoRaw);
  t.submitted();
  assert.equal(t.sample(sudoRaw), false);
  assert.equal(t.promptSeen(), true, '"Sorry, try again" prompt suppresses again');
});

test('a reader already running when the panel opens fails closed', () => {
  const t = new SecureInputTracker();
  assert.equal(t.sample(sudoRaw), true, 'first sample must assume a read is in progress');
});

test('returning to the shell prompt always releases', () => {
  const t = new SecureInputTracker();
  t.sample(sudoRaw);
  assert.equal(t.secure, true);
  assert.equal(t.sample(prompt), false);
});

test('an unreachable backend fails closed', () => {
  const t = new SecureInputTracker();
  t.sample(prompt);
  assert.equal(t.unknown(), true);
});
