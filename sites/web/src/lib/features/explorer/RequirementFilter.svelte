<script lang="ts">
  import { LogIn } from "@lucide/svelte";
  import { getSession } from "$lib/components/shell/session";
  import Button from "$lib/components/ui/Button.svelte";
  import Combobox from "$lib/components/ui/Combobox.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import type { Option } from "$lib/components/ui/types";
  import { entryYearOf } from "$lib/core/degree/hooks";
  import type { ReqRef } from "$lib/core/explorer/filters";
  import { defaultVersion } from "$lib/core/programs";
  import { flattenRequirements } from "$lib/core/requirement-match";
  import { catalog, resource } from "$lib/data";
  import { plans } from "$lib/state";

  let {
    value,
    onchange,
  }: {
    /** The requirement courses must count toward, or null. */
    value: ReqRef | null;
    onchange: (next: ReqRef | null) => void;
  } = $props();

  const session = getSession();
  const signedIn = $derived(session.status === "signed-in");
  const plan = $derived(plans.active);

  // Degree programs and gen eds are Stellic data, for CMU members only.
  const index = resource(() => (signedIn ? catalog.programsIndex() : null));
  const planVersions = resource(() => (signedIn && plan && plan.programs.length > 0 ? catalog.auditVersions(plan.programs) : null));

  // A program picked here before choosing one of its requirements; otherwise the filter's own program.
  let picked = $state<string | null>(null);
  $effect.pre(() => {
    if (value !== null) picked = null;
  });
  const program = $derived(picked ?? (value === null ? null : String(value.auditId)));
  const version = resource(() => (program === null ? null : catalog.program(Number(program))));

  const GEN_ED = /general education|gen ?ed/iu;

  const programOptions = $derived.by((): Option[] => {
    const entryYear = plan ? entryYearOf(plan.startTerm) : null;
    const seen = new Set<string>();
    const out: Option[] = [];
    const add = (option: Option): void => {
      if (seen.has(option.value)) return;
      seen.add(option.value);
      out.push(option);
    };
    // The plan's programs and what they bring in (gen eds, concentrations) first.
    const versions = planVersions.status === "ready" ? [...(planVersions.value?.values() ?? [])] : [];
    for (const v of versions)
      add({ value: String(v.auditId), label: v.programName, description: `In your degree plan, ${v.name}` });
    const programs = index.value?.programs ?? [];
    const genEds = programs.filter((p) => p.kind === "requirements" && GEN_ED.test(p.name));
    const rest = programs.filter((p) => !genEds.includes(p) && p.kind !== "requirements");
    for (const p of [...genEds.toSorted((a, b) => b.name.localeCompare(a.name)), ...rest]) {
      const v = defaultVersion(p, entryYear);
      if (v === null) continue;
      add({
        value: String(v.auditId),
        label: p.name,
        description: GEN_ED.test(p.name) ? `Gen ed, ${v.name}` : v.name,
        keywords: GEN_ED.test(p.name) ? "gen ed general education college" : p.kind,
      });
    }
    // The current filter's program, if it isn't one of the above.
    if (value !== null && !seen.has(String(value.auditId)) && version.value)
      add({ value: String(value.auditId), label: version.value.programName, description: version.value.name });
    return out;
  });

  const requirementOptions = $derived.by((): Option[] => {
    const v = version.status === "ready" ? version.value : undefined;
    if (v === undefined || String(v.auditId) !== program) return [];
    const ids = plan?.programs.map((p) => p.catalogId) ?? [];
    return flattenRequirements(v, ids)
      .filter((f) => !f.broad)
      .map((f) => ({
        value: String(f.nodeId),
        // Indented to show the tree in a native select.
        label: `${"\u00A0\u00A0\u00A0".repeat(f.depth)}${f.name}${f.target ? ` (${f.target})` : ""}`,
      }));
  });

  const nodeValue = $derived(value !== null && String(value.auditId) === program ? String(value.nodeId) : "");
</script>

{#if session.status === "loading"}
  <Spinner size="sm" label="Checking your session" />
{:else if !signedIn}
  <div class="flex flex-col items-start gap-2 rounded-field bg-paper px-3 py-3">
    <p class="text-body-2 text-ink">Sign in with your CMU account to filter by gen eds and degree requirements.</p>
    <Button size="sm" variant="primary" icon={LogIn} onclick={() => session.signIn()}>Sign in</Button>
  </div>
{:else if index.status === "forbidden"}
  <p class="text-body-2 text-muted">Requirements are only for CMU accounts.</p>
{:else if index.status === "error"}
  <p class="text-body-2 text-danger">Couldn't load degree programs. {index.error}</p>
{:else}
  <Combobox
    label="College gen eds or program"
    options={programOptions}
    placeholder={index.status === "loading" ? "Loading programs" : "Select college or program"}
    emptyText="No program matches"
    disabled={index.status === "loading"}
    bind:value={
      () => program,
      (next) => {
        picked = next;
        if (next === null || (value !== null && String(value.auditId) !== next)) onchange(null);
      }
    }
  />
  <Select
    label="Requirement"
    placeholder={program === null
      ? "Select gen ed or requirement"
      : version.status === "loading"
        ? "Loading requirements"
        : requirementOptions.length === 0
          ? "No course requirements to filter by"
          : "Select gen ed or requirement"}
    options={requirementOptions}
    disabled={program === null || requirementOptions.length === 0}
    bind:value={
      () => nodeValue,
      (next) => {
        if (program !== null && next !== "") onchange({ auditId: Number(program), nodeId: Number(next) });
      }
    }
  />
{/if}
