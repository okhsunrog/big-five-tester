#!/usr/bin/env python3
"""Experiment: Generate analysis in English, then translate to Russian."""

import asyncio
import json
import os
import time
from datetime import datetime

import httpx

# Profile data
PROFILE = {
    "domains": [
        {
            "domain": "Neuroticism",
            "raw": 66,
            "facets": [
                {"facet": "Anxiety", "raw": 14},
                {"facet": "Anger", "raw": 7},
                {"facet": "Depression", "raw": 9},
                {"facet": "SelfConsciousness", "raw": 10},
                {"facet": "Immoderation", "raw": 15},
                {"facet": "Vulnerability", "raw": 11},
            ],
        },
        {
            "domain": "Extraversion",
            "raw": 80,
            "facets": [
                {"facet": "Friendliness", "raw": 14},
                {"facet": "Gregariousness", "raw": 6},
                {"facet": "Assertiveness", "raw": 16},
                {"facet": "ActivityLevel", "raw": 15},
                {"facet": "ExcitementSeeking", "raw": 11},
                {"facet": "Cheerfulness", "raw": 18},
            ],
        },
        {
            "domain": "Openness",
            "raw": 98,
            "facets": [
                {"facet": "Imagination", "raw": 18},
                {"facet": "ArtisticInterests", "raw": 13},
                {"facet": "Emotionality", "raw": 16},
                {"facet": "Adventurousness", "raw": 15},
                {"facet": "Intellect", "raw": 17},
                {"facet": "Liberalism", "raw": 19},
            ],
        },
        {
            "domain": "Agreeableness",
            "raw": 98,
            "facets": [
                {"facet": "Trust", "raw": 18},
                {"facet": "Morality", "raw": 18},
                {"facet": "Altruism", "raw": 20},
                {"facet": "Cooperation", "raw": 19},
                {"facet": "Modesty", "raw": 7},
                {"facet": "Sympathy", "raw": 16},
            ],
        },
        {
            "domain": "Conscientiousness",
            "raw": 80,
            "facets": [
                {"facet": "SelfEfficacy", "raw": 16},
                {"facet": "Orderliness", "raw": 9},
                {"facet": "Dutifulness", "raw": 16},
                {"facet": "AchievementStriving", "raw": 17},
                {"facet": "SelfDiscipline", "raw": 10},
                {"facet": "Cautiousness", "raw": 12},
            ],
        },
    ]
}

USER_CONTEXT = "Danila, 24 years old, middle Rust developer working remotely, struggles with organization, planning, and sleep schedule."


def build_english_prompt(profile: dict, user_context: str) -> str:
    """Build analysis prompt in English."""
    scores = ""
    for domain in profile["domains"]:
        domain_pct = (domain["raw"] / 120) * 100
        scores += (
            f"\n## {domain['domain']} (score: {domain['raw']}/120, {domain_pct:.0f}%)\n"
        )
        for facet in domain["facets"]:
            facet_pct = (facet["raw"] / 20) * 100
            scores += f"- {facet['facet']}: {facet['raw']}/20 ({facet_pct:.0f}%)\n"

    return f"""You are a personality psychology expert specializing in the Big Five model. Analyze the IPIP-NEO-120 test results below and write a comprehensive, insightful personality profile.

# Test Results (IPIP-NEO-120)

Scores range from 24-120 for domains and 4-20 for facets.
- Low: below 40% 
- Neutral: 40-60%
- High: above 60%
{scores}

# User Context
"{user_context}"

Use this context to make the analysis personalized and relevant.

# Output Structure

Write a structured analysis with these sections:

## 1. Overview (1 paragraph)
Brief synthesis of the overall personality profile - what makes this person unique, key contrasts and patterns.

## 2. Analysis by Domain (5 sections)
For each Big Five domain:
- Overall interpretation of the domain score
- Notable facet patterns (especially extreme highs/lows and contrasts)
- How this manifests in behavior and life
- Interactions with other domains

## 3. Strengths (bullet list)
5-7 key strengths derived from the profile. Be specific, not generic.

## 4. Growth Areas (bullet list)  
3-5 potential challenges or areas for development. Frame constructively.

## 5. Synthesis (1 paragraph)
Integrative summary: what type of person emerges from this data, how different traits interact.

# Style Guidelines
1. Write in English. Use informal "you" to make it personal.
2. Be specific - reference actual percentages and facet names
3. Look for patterns: contrasts within domains, interactions between domains
4. Avoid generic statements that could apply to anyone
5. Use concrete behavioral examples where possible

Begin the analysis:"""


def build_translation_prompt(english_text: str) -> str:
    """Build prompt for translation to Russian."""
    return f"""Translate the following personality analysis from English to Russian.

Requirements:
1. Use informal "ты" form throughout
2. Keep psychological terms accurate but natural in Russian
3. Keep proper names and acronyms as-is: IPIP-NEO-120, Big Five, Rust
4. Preserve all markdown formatting (headers, bullet points, bold)
5. Make it sound natural in Russian, not like a literal translation
6. Keep all percentages and numbers as-is

Text to translate:

{english_text}"""


def log(msg: str):
    timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
    print(f"[{timestamp}] {msg}", flush=True)


async def query_model(
    client: httpx.AsyncClient,
    model: str,
    prompt: str,
    api_key: str,
    max_tokens: int = 8192,
) -> dict:
    """Query a model via OpenRouter."""
    start = time.time()

    response = await client.post(
        "https://openrouter.ai/api/v1/chat/completions",
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        },
        json={
            "model": model,
            "max_tokens": max_tokens,
            "messages": [{"role": "user", "content": prompt}],
        },
        timeout=300.0,
    )

    elapsed = time.time() - start

    if response.status_code != 200:
        return {
            "error": f"HTTP {response.status_code}: {response.text}",
            "time": elapsed,
        }

    data = response.json()
    content = data["choices"][0]["message"]["content"]
    usage = data.get("usage", {})

    return {
        "content": content,
        "time": elapsed,
        "input_tokens": usage.get("prompt_tokens", 0),
        "output_tokens": usage.get("completion_tokens", 0),
    }


async def main():
    api_key = os.environ.get("OPENROUTER_API_KEY")
    if not api_key:
        print("Error: OPENROUTER_API_KEY not set")
        return

    log("=" * 60)
    log("Translation Experiment: DeepSeek (EN) -> Gemini (RU)")
    log("=" * 60)

    # Models to test
    analysis_model = "deepseek/deepseek-v3.2"
    translation_models = [
        "google/gemini-2.5-flash-lite",
        "google/gemini-2.5-flash",
    ]

    english_prompt = build_english_prompt(PROFILE, USER_CONTEXT)
    log(f"English prompt: {len(english_prompt)} chars")

    async with httpx.AsyncClient() as client:
        # Step 1: Generate analysis in English
        log("")
        log(f"Step 1: Generating analysis with {analysis_model} (English)...")
        analysis_result = await query_model(
            client, analysis_model, english_prompt, api_key
        )

        if "error" in analysis_result:
            log(f"ERROR: {analysis_result['error']}")
            return

        english_analysis = analysis_result["content"]
        log(
            f"  Done in {analysis_result['time']:.1f}s, {analysis_result['output_tokens']} tokens"
        )
        log(f"  English analysis: {len(english_analysis)} chars")

        # Step 2: Translate with each model
        translation_prompt = build_translation_prompt(english_analysis)
        results = []

        for trans_model in translation_models:
            log("")
            log(f"Step 2: Translating with {trans_model}...")
            trans_result = await query_model(
                client, trans_model, translation_prompt, api_key
            )

            if "error" in trans_result:
                log(f"  ERROR: {trans_result['error']}")
                results.append({"model": trans_model, "error": trans_result["error"]})
            else:
                log(
                    f"  Done in {trans_result['time']:.1f}s, {trans_result['output_tokens']} tokens"
                )
                results.append(
                    {
                        "model": trans_model,
                        "content": trans_result["content"],
                        "time": trans_result["time"],
                        "tokens": trans_result["output_tokens"],
                    }
                )

    # Save results
    log("")
    log("=" * 60)
    log("RESULTS")
    log("=" * 60)

    output_dir = "model_outputs"
    os.makedirs(output_dir, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

    # Save English original
    with open(f"{output_dir}/translate_exp_{timestamp}_english.md", "w") as f:
        f.write(f"# Original Analysis (DeepSeek V3.2 - English)\n\n")
        f.write(f"**Time:** {analysis_result['time']:.1f}s\n")
        f.write(f"**Tokens:** {analysis_result['output_tokens']}\n\n")
        f.write("---\n\n")
        f.write(english_analysis)

    log(f"Saved English original")

    # Save translations
    for r in results:
        if "error" in r:
            continue
        model_name = r["model"].replace("/", "_")
        filename = f"{output_dir}/translate_exp_{timestamp}_{model_name}.md"
        with open(filename, "w") as f:
            f.write(f"# Translation by {r['model']}\n\n")
            f.write(f"**Analysis model:** {analysis_model}\n")
            f.write(f"**Analysis time:** {analysis_result['time']:.1f}s\n")
            f.write(f"**Translation time:** {r['time']:.1f}s\n")
            f.write(f"**Total time:** {analysis_result['time'] + r['time']:.1f}s\n")
            f.write(f"**Translation tokens:** {r['tokens']}\n\n")
            f.write("---\n\n")
            f.write(r["content"])
        log(f"Saved: {filename}")

    # Summary
    log("")
    log("Summary:")
    log(f"  DeepSeek analysis: {analysis_result['time']:.1f}s")
    for r in results:
        if "error" not in r:
            total = analysis_result["time"] + r["time"]
            log(f"  + {r['model']}: {r['time']:.1f}s (total: {total:.1f}s)")


if __name__ == "__main__":
    asyncio.run(main())
