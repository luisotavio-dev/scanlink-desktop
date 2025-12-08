import { spawn } from 'child_process';
import { platform } from 'os';

const isLinux = platform() === 'linux';

// No Linux, limpar variáveis GTK do Snap para evitar conflitos
const env = { ...process.env };

if (isLinux) {
  // Essas variáveis podem causar conflitos quando o VS Code é executado via Snap
  env.GTK_PATH = '';
  env.GTK_EXE_PREFIX = '';
  env.GIO_MODULE_DIR = '';
  env.LOCPATH = '';
}

const isWindows = platform() === 'win32';
const command = isWindows ? 'pnpm.cmd' : 'pnpm';

const child = spawn(command, ['tauri', 'dev'], {
  stdio: 'inherit',
  env,
  shell: true
});

child.on('close', (code) => {
  process.exit(code ?? 0);
});
