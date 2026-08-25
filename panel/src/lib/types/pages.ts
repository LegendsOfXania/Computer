import { FileText, Route } from "lucide-svelte";
import type { PageType } from "$lib/types/model";

export const PAGE_ICONS: Record<PageType, typeof Route> = {
  sequence: Route,
  static: FileText,
};
