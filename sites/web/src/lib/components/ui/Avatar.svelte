<script lang="ts" module>
  export type AvatarSize = "xs" | "sm" | "md" | "lg";

  /** Up to `count` initials: "Andrew Carnegie" -> "AC"; "Kosbie, David" -> "DK". */
  export function initials(name: string, count: 1 | 2 = 2): string {
    const [last = "", first = ""] = name.includes(",") ? name.split(",") : ["", name];
    const words = `${first} ${last}`.trim().split(/\s+/u).filter(Boolean);
    const picked = count === 1 ? words.slice(0, 1) : [words.at(0), words.length > 1 ? words.at(-1) : undefined];
    return picked
      .map((word) => word?.charAt(0) ?? "")
      .join("")
      .toUpperCase();
  }
</script>

<script lang="ts">
  import { User } from "@lucide/svelte";

  let {
    name,
    src,
    size = "sm",
    tone = "neutral",
    letters = 2,
    decorative = false,
    class: className = "",
  }: {
    /** Person's name, for the initials and the accessible name. Omit for the generic silhouette. */
    name?: string;
    /** Photo URL; initials show until it loads and if it fails. */
    src?: string;
    /** xs 24px (saved cards); sm 35px (course cards); md 48px (friend cards); lg 80px (profile). */
    size?: AvatarSize;
    /** neutral: readable initials on a light fill; soft: white initials on the pressed fill (Figma cards). */
    tone?: "neutral" | "soft";
    letters?: 1 | 2;
    /** Hide from assistive technology when the name is already next to it. */
    decorative?: boolean;
    class?: string;
  } = $props();

  let failed = $state(false);
  const sizes: Record<AvatarSize, string> = {
    xs: "size-6 text-label-md",
    sm: "size-[35px] text-[20px] leading-6 font-medium",
    md: "size-12 text-heading-6",
    lg: "size-20 text-heading-4",
  };
  const tones = { neutral: "bg-line text-ink", soft: "bg-pressed text-white" };
</script>

<span
  role={decorative ? undefined : "img"}
  aria-label={decorative ? undefined : (name ?? "Profile picture")}
  aria-hidden={decorative ? "true" : undefined}
  class="relative inline-grid shrink-0 place-items-center overflow-hidden rounded-full select-none {sizes[
    size
  ]} {tones[tone]} {className}"
>
  {#if src && !failed}
    <img {src} alt="" loading="lazy" class="size-full object-cover" onerror={() => (failed = true)} />
  {:else if name}
    <span aria-hidden="true">{initials(name, letters)}</span>
  {:else}
    <User aria-hidden="true" class="size-[55%] text-muted" />
  {/if}
</span>
