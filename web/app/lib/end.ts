// How a run ended, as its first line says it (plan section 1, screen 8). The live log's end and the report use the
// same words.
import type { EndView } from "../../shared/types/EndView";

export interface EndWords {
  title: string;
  text: string;
  icon: "green" | "amber" | "red";
}

export function endWords(end: EndView): EndWords {
  switch (end.kind) {
    case "finished":
      return { title: "Finished", text: "The run finished.", icon: "green" };
    case "stopped_by_steplet":
      return { title: "Steplet stopped the run", text: end.why, icon: "red" };
    case "stopped_by_you":
      return {
        title: "Interrupted",
        text: "You stopped the run. Its folder was reset, so you can run the file again.",
        icon: "amber",
      };
    case "model_refused":
      return { title: "The model refused", text: "The model refused the task, so the run did not finish.", icon: "amber" };
    case "did_not_finish":
      return { title: "Did not finish", text: "The run ended before the agent finished.", icon: "amber" };
  }
}
