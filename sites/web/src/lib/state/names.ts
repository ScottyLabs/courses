// Names for records joining a list that may already use them, such as a signed-out draft "Schedule 1" saved into an
// account that has a "Schedule 1" for the same semester.

/**
 * `name` when nobody in `taken` uses it. Otherwise a default name ("Schedule 1", with `base` "Schedule") becomes the
 * next free default ("Schedule 2"), and any other name gets a number ("Fall plan" to "Fall plan 2").
 */
export function distinctName(name: string, taken: readonly string[], base: string): string {
  const names = new Set(taken);
  if (!names.has(name)) return name;
  const rest = name.startsWith(`${base} `) ? name.slice(base.length + 1) : "";
  if (/^\d+$/u.test(rest)) {
    let n = taken.length + 1;
    while (names.has(`${base} ${n}`)) n += 1;
    return `${base} ${n}`;
  }
  let n = 2;
  while (names.has(`${name} ${n}`)) n += 1;
  return `${name} ${n}`;
}
