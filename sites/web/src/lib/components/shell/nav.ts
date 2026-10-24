import { CalendarDays, CircleUser, Clock, GraduationCap, Search, User } from "@lucide/svelte";
import type { IconComponent } from "../ui/types";

export interface NavItem {
  href: string;
  label: string;
  icon: IconComponent;
}

/** Main navigation, in header order. */
export const NAV_ITEMS: readonly NavItem[] = [
  { href: "/", label: "Courses", icon: Search },
  { href: "/instructors", label: "Instructors", icon: CircleUser },
  { href: "/schedules", label: "Schedules", icon: Clock },
  { href: "/plan", label: "Plan", icon: GraduationCap },
  { href: "/finals", label: "Finals", icon: CalendarDays },
  { href: "/profile", label: "Profile", icon: User },
];

/** Whether `item` is the current section: course pages belong to Courses, invites to Profile. */
export function isActive(item: NavItem, pathname: string): boolean {
  if (item.href === "/") return pathname === "/" || pathname.startsWith("/course/");
  if (item.href === "/profile" && pathname.startsWith("/invite/")) return true;
  return pathname === item.href || pathname.startsWith(`${item.href}/`);
}
