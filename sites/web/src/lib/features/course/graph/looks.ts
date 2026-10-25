// How the requisite graph draws each status: the pills' fill and icon, the brackets' color, and the legend's words.
// Met and waived are green, in progress and planned yellow (as the requisite chips elsewhere), a grade below the
// minimum red, and not met plain with an x, so no status is told by color alone.
import {
  ArrowRight,
  BadgeCheck,
  Check,
  Clock,
  Info,
  LockOpen,
  TriangleAlert,
  X,
} from "@lucide/svelte";
import type { IconComponent } from "$lib/components/ui/types";
import type { LinkState, PillStatus } from "./types";

export interface Look {
  label: string;
  /** What it means, for the legend. */
  help: string;
  icon: IconComponent;
  /** The pill's border and fill. */
  box: string;
  /** The icon's color. */
  tint: string;
}

export const PILL_LOOKS: Record<PillStatus, Look> = {
  met: {
    label: "Met",
    help: "Taken, or credit from transfer or an exam",
    icon: Check,
    box: "border-transparent bg-ok-bg",
    tint: "text-ok",
  },
  "in-progress": {
    label: "In progress",
    help: "Taking it this semester",
    icon: Clock,
    box: "border-transparent bg-warn-bg",
    tint: "text-warn",
  },
  planned: {
    label: "Planned",
    help: "In a schedule or your degree plan",
    icon: ArrowRight,
    box: "border-transparent bg-warn-bg",
    tint: "text-warn",
  },
  missing: {
    label: "Not met",
    help: "Not taken or planned",
    icon: X,
    box: "border-line-strong bg-surface",
    tint: "text-muted",
  },
  waived: {
    label: "Waived",
    help: "Placed out, or requisites waived in your record",
    icon: BadgeCheck,
    box: "border-transparent bg-ok-bg",
    tint: "text-ok",
  },
  "grade-low": {
    label: "Grade too low",
    help: "Taken, but below the minimum grade",
    icon: TriangleAlert,
    box: "border-transparent bg-danger-bg",
    tint: "text-danger",
  },
};

/** A course with nothing to say about it. */
export const PLAIN_BOX = "border-pressed bg-surface";

/** An unlocked course the student could take next. */
export const READY: Look = {
  label: "Could take next",
  help: "Its other prerequisites are met",
  icon: LockOpen,
  box: "border-accent bg-surface",
  tint: "text-accent-ink",
};

/** The pill's look: its status, else ready, else plain. */
export function pillLook(status: PillStatus | undefined, ready: boolean): Look | null {
  if (status !== undefined) return PILL_LOOKS[status];
  return ready ? READY : null;
}

export interface StateLook {
  /** The bracket's and its lines' color. */
  stroke: string;
  /** The label's border and text. */
  label: string;
  icon: IconComponent | null;
}

const STATE_LOOKS: Record<LinkState, StateLook> = {
  met: { stroke: "stroke-ok", label: "border-ok text-ok", icon: Check },
  waived: { stroke: "stroke-ok", label: "border-ok text-ok", icon: BadgeCheck },
  planned: { stroke: "stroke-warn", label: "border-warn text-warn", icon: ArrowRight },
  missing: { stroke: "stroke-line-strong", label: "border-line-strong text-muted", icon: X },
  unknown: { stroke: "stroke-line-strong", label: "border-line-strong text-muted", icon: Info },
};

const PLAIN_STATE: StateLook = {
  stroke: "stroke-line-strong",
  label: "border-line-strong text-ink",
  icon: null,
};

/** Bracket and label colors for a group's state; plain for groups that aren't requirements. */
export function stateLook(state: LinkState | undefined): StateLook {
  return state === undefined ? PLAIN_STATE : STATE_LOOKS[state];
}

/** Satisfied lines are drawn heavier, so the way to the middle course stands out. */
export function isSatisfied(state: LinkState | undefined): boolean {
  return state === "met" || state === "waived" || state === "planned";
}
