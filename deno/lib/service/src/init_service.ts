import { FenvContext, io, Shell } from '@fenv/lib';

export function showInitInstructions(
  context: FenvContext,
  shell: Shell,
): Promise<void> {
  switch (shell) {
    case Shell.BASH:
      return bash();
    case Shell.ZSH:
      return zsh();
    case Shell.FISH:
      return fish();
  }

  async function bash(): Promise<void> {
    await io.writeText(context.stdout, initOutputBash);
  }

  async function zsh(): Promise<void> {
    await io.writeText(context.stdout, initOutputZsh);
  }

  async function fish(): Promise<void> {
    await io.writeText(context.stdout, initOutputFish);
  }
}

const initOutputZsh = `# Load fenv automatically by appending the following to
# ~/.zprofile (for login shells)
# and ~/.zshrc (for interactive shells) :

export FENV_ROOT="$HOME/.fenv"
command -v fenv >/dev/null || export PATH="$FENV_ROOT/bin:$PATH"
eval "$(fenv init -)"

# Restart your shell for the changes to take effect:

exec $SHELL -l

`;

const initOutputBash = `# Load fenv automatically by appending the following to
# ~/.bash_profile if it exists, otherwise ~/.profile (for login shells)
# and ~/.bashrc (for interactive shells) :

export FENV_ROOT="$HOME/.fenv"
command -v fenv >/dev/null || export PATH="$FENV_ROOT/bin:$PATH"
eval "$(fenv init -)"

# Restart your shell for the changes to take effect:

exec $SHELL -l

`;

const initOutputFish = `# Add fenv executable to PATH by running
# the following interactively:

set -Ux FENV_ROOT $HOME/.fenv
fish_add_path $FENV_ROOT/bin

# Load fenv automatically by appending
# the following to ~/.config/fish/conf.d/fenv.fish:

fenv init - | source

# Restart your shell for the changes to take effect:

exec $SHELL -l

`;
