import type { ReactNode } from "react";
import { Tabs } from "@base-ui/react/tabs";
import { Compass, Files } from "lucide-react";

export function WorkbenchTabs({ orient, source }: { orient: ReactNode; source: ReactNode }) {
  return (
    <Tabs.Root defaultValue="orient" className="tabs-root">
      <Tabs.List className="tabs-list">
        <Tabs.Tab value="orient" className="tabs-tab"><Compass size={15} />Orient</Tabs.Tab>
        <Tabs.Tab value="source" className="tabs-tab"><Files size={15} />Execute / Source</Tabs.Tab>
        <Tabs.Indicator className="tabs-indicator" />
      </Tabs.List>
      <Tabs.Panel value="orient" className="tabs-panel">{orient}</Tabs.Panel>
      <Tabs.Panel value="source" className="tabs-panel">{source}</Tabs.Panel>
    </Tabs.Root>
  );
}
