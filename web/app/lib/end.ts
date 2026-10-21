// The outcome icon of a run's end. Rust gives the words and the colour (`EndWords`); the screens only draw them.

export type OutcomeIcon = "green" | "amber" | "red";

export const outcomeIcon = (icon: string): OutcomeIcon => (icon === "green" || icon === "red" ? icon : "amber");
