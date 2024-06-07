import { $Type } from '@david/dax';

export function getPpidExecutablePath($: $Type, ppid: number): Promise<string> {
  return $`bash -c 'ps -p ${ppid} -o args='`.quiet('stderr').text();
}
