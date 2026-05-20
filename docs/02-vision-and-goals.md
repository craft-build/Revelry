<!-- Revelry Design Specification v2.0 — Section 2 -->
<!-- Last updated: 2026-05-20 -->

# Product Vision and Goals

## 2.1 Vision Statement

To create the most refined, privacy-respecting, and creatively empowering AI roleplay experience on the desktop, built as a native GNOME citizen using pure Rust, where users can bring any character to life through conversation and storytelling, supported by a beautifully minimal interface that disappears into the background of their imagination.

## 2.2 Target Audience

Revelry targets several distinct user segments, each with unique needs and expectations. Creative writers and novelists form a primary audience, seeking AI collaborators that can help them explore character voices, test dialogue, and develop storylines through interactive conversation. These users value depth, consistency, and the ability to maintain long narrative contexts across many sessions. The native desktop experience with responsive GTK4 widgets ensures that long writing sessions remain fluid and distraction-free.

Tabletop roleplaying game enthusiasts represent another key segment, using AI characters as dungeon masters, NPCs, or fellow party members. They require structured scenario management, world-building tools through the lorebook system, and the ability to maintain consistent rules and settings across campaigns. Revelry's local-first architecture ensures their custom worlds remain private and always accessible, while rig's Ollama integration allows them to run the entire stack locally for complete data sovereignty.

Casual roleplay hobbyists who currently use platforms like Character.ai or SpicyChat form the broadest potential user base. These users want an easy on-ramp with intuitive character creation, seamless conversation flow, and the ability to explore diverse character interactions. They are drawn by the promise of uncensored, private interactions that respect their creative freedom without corporate content policy restrictions. The GNOME-style progressive disclosure ensures newcomers see a simple chat interface while power users discover advanced features organically.

AI researchers and hobbyists who run local models through Ollama, LM Studio, vLLM, or similar tools need a polished front-end that supports their self-hosted infrastructure. Revelry's rig-based integration means these users can configure any OpenAI-compatible endpoint and enjoy the same experience as cloud-based users, with full data sovereignty. The rig framework's native Ollama client eliminates the need for a separate OpenAI-compatible translation layer, simplifying configuration and improving streaming performance for local models.

## 2.3 Key Objectives

Deliver a native GNOME desktop experience using GTK4 and libadwaita, with Adwaita styling, GNOME HIG compliance, and eligibility for GNOME Circle inclusion. Implement a comprehensive character card system compatible with the SillyTavern/CharacterCardV2 specification for maximum interoperability. Support any AI provider through the rig framework's unified abstraction, which provides native client implementations for over twenty providers including OpenAI, Anthropic, Ollama, Gemini, Cohere, Mistral, Groq, DeepSeek, and OpenRouter. Apply GNOME design principles through native GTK4/libadwaita widgets to create a distraction-free, content-first UI. Ensure all user data is stored locally with no mandatory cloud synchronization or account creation. Provide advanced features including conversation branching, lorebooks, prompt presets, user personas, character grouping, and regex scripting for power users.
