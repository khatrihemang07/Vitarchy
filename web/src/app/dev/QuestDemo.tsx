import { useState } from "react";
import { Card, QuestList } from "@/design";
import { quests as initial } from "./sample-data";

export function QuestDemo() {
  const [quests, setQuests] = useState(initial);
  const toggle = (id: string) => {
    setQuests((qs) => qs.map((q) => (q.id === id ? { ...q, done: !q.done } : q)));
  };
  return (
    <Card className="p-2">
      <QuestList quests={quests} onToggle={toggle} />
    </Card>
  );
}
