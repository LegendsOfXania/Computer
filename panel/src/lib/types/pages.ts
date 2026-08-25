import { FileText, Route } from "lucide-svelte";

export type PageType = "sequence" | "static";

export interface Page {
  id: string;
  name: string;
  page_type: PageType;
  priority: number;
}

export const PAGE_ICONS: Record<PageType, typeof Route> = {
  sequence: Route,
  static: FileText,
};
