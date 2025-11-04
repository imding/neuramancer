use dioxus::prelude::*;

const HERO_CSS: Asset = asset!("/assets/styling/hero.css");

#[component]
pub fn Hero() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: HERO_CSS }

        div { id: "hero",

            div { id: "pitch",

                h1 { "Neuramancy" }
                p {
                    "Exhume forgotten wishpers from the void and chain them with purpose, summon spirits of fellow neuramancers to enrich your tome, and resurrect knowledge as a sentient grimoire."
                }
            }

            div { id: "cta",
                button { id: "sign-up", "Sign-up" }
                button { id: "watch-video", "Watch video" }
            }
        }
    }
}
