use dioxus::prelude::*;

#[component]
pub fn Hero() -> Element {
    rsx! {
        div { class: "grid place-content-center text-center gap-4",

            div { class: "w-[65ch]",

                h1 { "Neuramancy" }
                p {
                    "Exhume forgotten wishpers from the void and chain them with purpose, summon spirits of fellow neuramancers to enrich your tome, and resurrect knowledge as a sentient grimoire."
                }
            }

            div { class: "grid grid-flow-col gap-4",
                button { class: "text-xl p-4 border-none rounded-full cursor-pointer bg-[oklch(70.2%_0.183_293.541)] text-[oklch(25.7%_0.09_281.288)]",
                    "Sign-up"
                }
                button { class: "text-xl p-4 border-none rounded-full cursor-pointer",
                    "Watch video"
                }
            }
        }
    }
}
