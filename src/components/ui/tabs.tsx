import type { ReactNode } from "react";
import { Tabs } from "@base-ui/react/tabs";
import { Compass, FileCheck2, FlaskConical, Play } from "lucide-react";

export function WorkbenchTabs({ orient, execute, capture, review }: { orient: ReactNode; execute: ReactNode; capture: ReactNode; review: ReactNode }) {
  return (
    <Tabs.Root defaultValue="orient" className="tabs-root">
      <Tabs.List className="tabs-list">
        <Tabs.Tab value="orient" className="tabs-tab"><Compass size={15} />Orient</Tabs.Tab>
        <Tabs.Tab value="execute" className="tabs-tab"><Play size={15} />Execute</Tabs.Tab>
        <Tabs.Tab value="capture" className="tabs-tab"><FlaskConical size={15} />Capture</Tabs.Tab>
        <Tabs.Tab value="review" className="tabs-tab"><FileCheck2 size={15} />Review draft</Tabs.Tab>
        <Tabs.Indicator className="tabs-indicator" />
      </Tabs.List>
      <Tabs.Panel value="orient" className="tabs-panel">{orient}</Tabs.Panel>
      <Tabs.Panel value="execute" className="tabs-panel">{execute}</Tabs.Panel>
      <Tabs.Panel value="capture" className="tabs-panel">{capture}</Tabs.Panel>
      <Tabs.Panel value="review" className="tabs-panel">{review}</Tabs.Panel>
    </Tabs.Root>
  );
}
