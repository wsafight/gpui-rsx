/** Patch releases share the editor protocol within one major.minor release line. */
function releaseLine(version: string): string | undefined {
  return /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/
    .exec(version)?.slice(1, 3).join('.');
}

export function compatibleServerVersion(server: string, extension: string): boolean {
  const line = releaseLine(extension);
  return line !== undefined && releaseLine(server) === line;
}

/** Cargo selects the latest published patch without crossing the protocol boundary. */
export function serverVersionRequirement(extension: string): string {
  const line = releaseLine(extension);
  if (line === undefined) throw new Error(`Invalid GPUI RSX extension version: ${extension}`);
  return `~${line}.0`;
}
