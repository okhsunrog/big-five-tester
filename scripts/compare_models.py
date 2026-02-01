#!/usr/bin/env python3
"""Compare different LLM models for Big Five personality analysis via OpenRouter."""

import asyncio
import json
import os
import sys
import time
from dataclasses import asdict, dataclass
from datetime import datetime

import httpx

# Your test profile data
PROFILE = {
    "domains": [
        {
            "domain": "Neuroticism",
            "raw": 66,
            "level": "Neutral",
            "facets": [
                {"facet": "Anxiety", "raw": 14, "level": "Neutral"},
                {"facet": "Anger", "raw": 7, "level": "Low"},
                {"facet": "Depression", "raw": 9, "level": "Low"},
                {"facet": "SelfConsciousness", "raw": 10, "level": "Neutral"},
                {"facet": "Immoderation", "raw": 15, "level": "High"},
                {"facet": "Vulnerability", "raw": 11, "level": "Neutral"},
            ],
        },
        {
            "domain": "Extraversion",
            "raw": 80,
            "level": "Neutral",
            "facets": [
                {"facet": "Friendliness", "raw": 14, "level": "Neutral"},
                {"facet": "Gregariousness", "raw": 6, "level": "Low"},
                {"facet": "Assertiveness", "raw": 16, "level": "High"},
                {"facet": "ActivityLevel", "raw": 15, "level": "High"},
                {"facet": "ExcitementSeeking", "raw": 11, "level": "Neutral"},
                {"facet": "Cheerfulness", "raw": 18, "level": "High"},
            ],
        },
        {
            "domain": "Openness",
            "raw": 98,
            "level": "High",
            "facets": [
                {"facet": "Imagination", "raw": 18, "level": "High"},
                {"facet": "ArtisticInterests", "raw": 13, "level": "Neutral"},
                {"facet": "Emotionality", "raw": 16, "level": "High"},
                {"facet": "Adventurousness", "raw": 15, "level": "High"},
                {"facet": "Intellect", "raw": 17, "level": "High"},
                {"facet": "Liberalism", "raw": 19, "level": "High"},
            ],
        },
        {
            "domain": "Agreeableness",
            "raw": 98,
            "level": "High",
            "facets": [
                {"facet": "Trust", "raw": 18, "level": "High"},
                {"facet": "Morality", "raw": 18, "level": "High"},
                {"facet": "Altruism", "raw": 20, "level": "High"},
                {"facet": "Cooperation", "raw": 19, "level": "High"},
                {"facet": "Modesty", "raw": 7, "level": "Low"},
                {"facet": "Sympathy", "raw": 16, "level": "High"},
            ],
        },
        {
            "domain": "Conscientiousness",
            "raw": 80,
            "level": "Neutral",
            "facets": [
                {"facet": "SelfEfficacy", "raw": 16, "level": "High"},
                {"facet": "Orderliness", "raw": 9, "level": "Low"},
                {"facet": "Dutifulness", "raw": 16, "level": "High"},
                {"facet": "AchievementStriving", "raw": 17, "level": "High"},
                {"facet": "SelfDiscipline", "raw": 10, "level": "Neutral"},
                {"facet": "Cautiousness", "raw": 12, "level": "Neutral"},
            ],
        },
    ]
}

# User context for personalized analysis
USER_CONTEXT = """Данила, 24 года, middle rust разработчик на удалёнке, проблемы с организованностью, планированием, распорядком сна."""

# Models to compare (OpenRouter model IDs)
MODELS = [
    "anthropic/claude-haiku-4.5",
]


def log(msg: str, level: str = "INFO"):
    """Print a timestamped log message."""
    timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
    print(f"[{timestamp}] [{level}] {msg}", flush=True)


def build_prompt(
    profile: dict, lang: str = "en", user_context: str | None = None
) -> str:
    """Build the analysis prompt from profile data."""
    scores = ""
    for domain in profile["domains"]:
        domain_pct = (domain["raw"] / 120) * 100
        scores += (
            f"\n## {domain['domain']} (score: {domain['raw']}/120, {domain_pct:.0f}%)\n"
        )
        for facet in domain["facets"]:
            facet_pct = (facet["raw"] / 20) * 100
            scores += f"- {facet['facet']}: {facet['raw']}/20 ({facet_pct:.0f}%)\n"

    lang_instruction = (
        'Write in Russian using informal "ты" form.\n'
        "Keep proper names and acronyms in original form: IPIP-NEO-120, Big Five, etc.\n"
        "Do not use casual English words where Russian equivalents exist."
        if lang == "ru"
        else 'Write in English. Use informal "you" to make it personal.'
    )

    context_section = ""
    if user_context and user_context.strip():
        context_section = f"""
# User-Provided Context

The person has shared the following about themselves:
"{user_context.strip()}"

Use this context to make the analysis more personalized and relevant. Reference specific details from their description where they align with or contrast with the test results.
"""

    return f"""You are a personality psychology expert specializing in the Big Five model. Analyze the IPIP-NEO-120 test results below and write a comprehensive, insightful personality profile.

# Test Results (IPIP-NEO-120)

Scores range from 24-120 for domains (5 domains x 24 questions each) and 4-20 for facets (6 facets per domain x 4 questions each).
- Low: below 40% 
- Neutral: 40-60%
- High: above 60%
{scores}
{context_section}
# Output Structure

Write a structured analysis with these sections:

## 1. Overview (1 paragraph)
Brief synthesis of the overall personality profile - what makes this person unique, key contrasts and patterns.

## 2. Analysis by Domain (5 sections)
For each of the Big Five domains, provide:
- Overall interpretation of the domain score
- Notable facet patterns (especially extreme highs/lows and interesting contrasts)
- How this manifests in behavior and life
- Interactions with other domains

## 3. Strengths (bullet list)
5-7 key strengths derived from the profile. Be specific, not generic.

## 4. Growth Areas (bullet list)  
3-5 potential challenges or areas for development. Frame constructively.

## 5. Synthesis (1 paragraph)
Integrative summary: what type of person emerges from this data, how different traits interact.

# Style Guidelines

1. {lang_instruction}
2. Be specific - reference actual percentages and facet names
3. Look for patterns: contrasts within domains, interactions between domains
4. Avoid generic statements that could apply to anyone
5. Use concrete behavioral examples where possible
6. Be balanced - acknowledge complexity, avoid oversimplification

Begin the analysis:"""


@dataclass
class ModelResult:
    model: str
    response: str
    time_seconds: float
    input_tokens: int
    output_tokens: int
    total_tokens: int
    error: str | None = None


async def query_model(
    client: httpx.AsyncClient,
    model: str,
    prompt: str,
    api_key: str,
) -> ModelResult:
    """Query a single model via OpenRouter."""
    log(f"Starting request to {model}")
    start = time.time()

    try:
        log(f"  Sending POST request...")
        response = await client.post(
            "https://openrouter.ai/api/v1/chat/completions",
            headers={
                "Authorization": f"Bearer {api_key}",
                "Content-Type": "application/json",
                "HTTP-Referer": "https://bigfive.okhsunrog.ru",
                "X-Title": "Big Five Model Comparison",
            },
            json={
                "model": model,
                "max_tokens": 8192,
                "messages": [{"role": "user", "content": prompt}],
            },
            timeout=300.0,
        )

        elapsed = time.time() - start
        log(f"  Response received: HTTP {response.status_code} in {elapsed:.1f}s")

        if response.status_code != 200:
            error_text = response.text
            log(f"  ERROR: {error_text}", "ERROR")
            return ModelResult(
                model=model,
                response="",
                time_seconds=elapsed,
                input_tokens=0,
                output_tokens=0,
                total_tokens=0,
                error=f"HTTP {response.status_code}: {error_text}",
            )

        data = response.json()

        # Log raw response structure for debugging
        log(f"  Response keys: {list(data.keys())}")

        content = data["choices"][0]["message"]["content"]
        usage = data.get("usage", {})

        input_tokens = usage.get("prompt_tokens", 0)
        output_tokens = usage.get("completion_tokens", 0)
        total_tokens = usage.get("total_tokens", input_tokens + output_tokens)

        log(
            f"  Success! Tokens: {input_tokens} in / {output_tokens} out / {total_tokens} total"
        )
        log(f"  Response length: {len(content)} chars")

        return ModelResult(
            model=model,
            response=content,
            time_seconds=elapsed,
            input_tokens=input_tokens,
            output_tokens=output_tokens,
            total_tokens=total_tokens,
        )

    except httpx.TimeoutException:
        elapsed = time.time() - start
        log(f"  TIMEOUT after {elapsed:.1f}s", "ERROR")
        return ModelResult(
            model=model,
            response="",
            time_seconds=elapsed,
            input_tokens=0,
            output_tokens=0,
            total_tokens=0,
            error="Request timed out (300s)",
        )
    except Exception as e:
        elapsed = time.time() - start
        log(f"  EXCEPTION: {type(e).__name__}: {e}", "ERROR")
        return ModelResult(
            model=model,
            response="",
            time_seconds=elapsed,
            input_tokens=0,
            output_tokens=0,
            total_tokens=0,
            error=f"{type(e).__name__}: {e}",
        )


async def main():
    log("=" * 60)
    log("Big Five Model Comparison Script")
    log("=" * 60)

    api_key = os.environ.get("OPENROUTER_API_KEY")
    if not api_key:
        log("OPENROUTER_API_KEY environment variable not set!", "ERROR")
        log("Get your key at https://openrouter.ai/keys")
        sys.exit(1)

    log(f"API key found: {api_key[:12]}...{api_key[-4:]}")
    log(f"Models to test: {len(MODELS)}")
    for m in MODELS:
        log(f"  - {m}")

    prompt = build_prompt(PROFILE, lang="ru", user_context=USER_CONTEXT)
    log(f"Prompt built: {len(prompt)} chars")

    log("")
    log("=" * 60)
    log("Starting model queries...")
    log("=" * 60)

    results: list[ModelResult] = []
    total_start = time.time()

    async with httpx.AsyncClient() as client:
        for i, model in enumerate(MODELS, 1):
            log("")
            log(f"[{i}/{len(MODELS)}] {model}")
            log("-" * 40)
            result = await query_model(client, model, prompt, api_key)
            results.append(result)

            if result.error:
                log(f"  FAILED: {result.error}", "ERROR")
            else:
                log(
                    f"  COMPLETED: {result.time_seconds:.1f}s, {result.output_tokens} tokens"
                )

    total_time = time.time() - total_start

    log("")
    log("=" * 60)
    log("RESULTS SUMMARY")
    log("=" * 60)
    log(f"Total time: {total_time:.1f}s")
    log("")

    # Print summary table
    print(
        "\n{:<35} {:>8} {:>8} {:>10} {:>10}".format(
            "Model", "Time(s)", "Out Tok", "Total Tok", "Status"
        )
    )
    print("-" * 75)

    for r in results:
        status = "OK" if not r.error else "FAILED"
        print(
            "{:<35} {:>8.1f} {:>8} {:>10} {:>10}".format(
                r.model[:35], r.time_seconds, r.output_tokens, r.total_tokens, status
            )
        )

    # Save all results to a single comprehensive markdown file
    output_dir = "model_outputs"
    os.makedirs(output_dir, exist_ok=True)

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = f"{output_dir}/comparison_{timestamp}.md"

    with open(output_file, "w", encoding="utf-8") as f:
        f.write("# Big Five Model Comparison Results\n\n")
        f.write(f"**Date:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write(f"**Total Time:** {total_time:.1f}s\n")
        f.write(f"**Language:** Russian\n\n")

        f.write("## User Context\n\n")
        f.write(f"> {USER_CONTEXT}\n\n")

        f.write("## Summary Table\n\n")
        f.write("| Model | Time | Output Tokens | Status |\n")
        f.write("|-------|------|---------------|--------|\n")
        for r in results:
            status = "OK" if not r.error else f"FAILED: {r.error[:30]}..."
            f.write(
                f"| {r.model} | {r.time_seconds:.1f}s | {r.output_tokens} | {status} |\n"
            )

        f.write("\n---\n\n")

        for r in results:
            f.write(f"# {r.model}\n\n")
            f.write(f"- **Time:** {r.time_seconds:.1f}s\n")
            f.write(f"- **Input Tokens:** {r.input_tokens}\n")
            f.write(f"- **Output Tokens:** {r.output_tokens}\n")
            f.write(f"- **Total Tokens:** {r.total_tokens}\n")

            if r.error:
                f.write(f"- **Error:** {r.error}\n")

            f.write("\n## Response\n\n")
            if r.response:
                f.write(r.response)
            else:
                f.write("*No response (error occurred)*")
            f.write("\n\n---\n\n")

    log("")
    log(f"Full results saved to: {output_file}")

    # Also save as JSON for programmatic access
    json_file = f"{output_dir}/comparison_{timestamp}.json"
    with open(json_file, "w", encoding="utf-8") as f:
        json.dump(
            {
                "timestamp": datetime.now().isoformat(),
                "total_time_seconds": total_time,
                "user_context": USER_CONTEXT,
                "models": MODELS,
                "results": [asdict(r) for r in results],
            },
            f,
            ensure_ascii=False,
            indent=2,
        )

    log(f"JSON results saved to: {json_file}")


if __name__ == "__main__":
    asyncio.run(main())
