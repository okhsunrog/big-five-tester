#!/usr/bin/env python3
"""Test 6 different analysis pipelines with final prompts."""

import asyncio
import os
import time
from datetime import datetime

import httpx

# Models
OPUS = "anthropic/claude-opus-4"
QWEN = "qwen/qwen3-235b-a22b-2507"
DEEPSEEK = "deepseek/deepseek-chat-v3-0324"
GEMINI_TRANSLATE = "google/gemini-2.5-flash-lite"

# Test profile
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

USER_CONTEXT = "Данила, 24 года, middle rust разработчик на удалёнке, проблемы с организованностью, планированием, распорядком сна."


def format_scores(profile: dict) -> str:
    """Format profile scores for the prompt."""
    scores = ""
    for domain in profile["domains"]:
        domain_pct = (domain["raw"] / 120) * 100
        scores += f"\n## {domain['domain']} ({domain['raw']}/120, {domain_pct:.0f}%)\n"
        for facet in domain["facets"]:
            facet_pct = (facet["raw"] / 20) * 100
            scores += f"- {facet['facet']}: {facet['raw']}/20 ({facet_pct:.0f}%)\n"
    return scores


# Prompts from Rust app (prompts.rs)
def prompt_russian(scores: str, context: str) -> str:
    context_section = f"**О человеке:** {context}\n" if context else ""
    return f"""Big Five (IPIP-NEO-120). Домены 24-120, фасеты 4-20. Низкий <40%, средний 40-60%, высокий >60%.

{scores}
{context_section}
Напиши психологический портрет:

## Обзор
Уникальность профиля, главные паттерны и контрасты.

## Нейротизм | Экстраверсия | Открытость | Доброжелательность | Сознательность
Каждый домен: общий балл → ключевые фасеты → как проявляется в жизни.

## Сильные стороны
5-6 конкретных преимуществ. Ссылайся на фасеты. Не общие фразы.

## Слабые стороны
3-4 реальных проблемы. Честно, но конструктивно.

## Рекомендации
5-6 практических действий:
- Использовать сильные стороны как ресурс
- Компенсировать слабости конкретными шагами
- Учитывать контекст жизни
- Давать выполнимые советы, не абстракции

## Итог
Тип личности, взаимодействие черт, ключевой вывод.

Стиль: русский, на "ты", конкретика (% и фасеты), без воды."""


def prompt_english(scores: str, context: str) -> str:
    context_section = f"**About the person:** {context}\n" if context else ""
    return f"""Big Five (IPIP-NEO-120). Domains 24-120, facets 4-20. Low <40%, neutral 40-60%, high >60%.

{scores}
{context_section}
Write a psychological profile:

## Overview
Profile uniqueness, main patterns and contrasts.

## Neuroticism | Extraversion | Openness | Agreeableness | Conscientiousness
Each domain: overall score → key facets → how it manifests in life.

## Strengths
5-6 specific advantages. Reference facets. No generic statements.

## Weaknesses
3-4 real challenges. Honest but constructive.

## Recommendations
5-6 practical actions:
- Use strengths as resources
- Compensate weaknesses with specific steps
- Consider life context
- Give actionable advice, not abstractions

## Conclusion
Personality type, trait interactions, key takeaway.

Style: English, use "you", specific (% and facets), no fluff."""


def prompt_chinese(scores: str, context: str) -> str:
    context_section = f"**关于此人:** {context}\n" if context else ""
    return f"""大五人格 (IPIP-NEO-120)。领域24-120分，方面4-20分。低 <40%，中 40-60%，高 >60%。

{scores}
{context_section}
撰写心理画像：

## 概述
人格特征的独特性，主要模式和对比。

## 神经质 | 外向性 | 开放性 | 宜人性 | 尽责性
每个维度：总分 → 关键方面 → 生活中的表现。

## 优势
5-6个具体优势。引用方面分数。避免泛泛之谈。

## 弱点
3-4个真实问题。诚实但建设性。

## 建议
5-6个实用行动：
- 利用优势作为资源
- 用具体步骤弥补弱点
- 考虑生活背景
- 给出可执行的建议，而非抽象概念

## 总结
人格类型，特质互动，核心结论。

风格：中文，使用"你"，具体（%和方面），无废话。"""


def translation_prompt(text: str, source_lang: str) -> str:
    """Translation prompt from source language to Russian."""
    source_name = {"en": "English", "zh": "Chinese"}[source_lang]
    return f"""Translate this personality analysis from {source_name} to Russian.

Requirements:
1. Use informal "ты" form in Russian.
2. Psychological terms should sound natural in Russian
3. Keep proper names and acronyms as-is: IPIP-NEO-120, Big Five, Rust
4. Preserve markdown formatting
5. Translation should sound natural, not literal
6. Preserve all details and nuances from the original

Text to translate:

{text}"""


async def call_api(
    client: httpx.AsyncClient,
    api_key: str,
    model: str,
    prompt: str,
    max_tokens: int = 8192,
) -> tuple[str, float, int, float]:
    """Call API. Returns (response, time, tokens, cost)."""
    start = time.time()

    resp = await client.post(
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

    if resp.status_code != 200:
        raise Exception(f"HTTP {resp.status_code}: {resp.text[:200]}")

    data = resp.json()
    content = data["choices"][0]["message"]["content"]
    tokens = data.get("usage", {}).get("total_tokens", 0)
    cost = data.get("usage", {}).get("cost", 0) or 0

    return content, elapsed, tokens, cost


async def run_pipeline(
    client: httpx.AsyncClient,
    api_key: str,
    name: str,
    analysis_model: str,
    prompt: str,
    translate: bool = False,
    source_lang: str | None = None,
) -> dict:
    """Run a single pipeline."""
    print(f"\n{'=' * 60}")
    print(f"Pipeline: {name}")
    print(f"Model: {analysis_model}")
    print(f"Translate: {translate}")
    print(f"{'=' * 60}")

    result = {
        "name": name,
        "model": analysis_model,
        "translate": translate,
    }

    try:
        # Step 1: Analysis
        print("  Generating analysis...")
        analysis, analysis_time, analysis_tokens, analysis_cost = await call_api(
            client, api_key, analysis_model, prompt
        )

        result["analysis_time"] = analysis_time
        result["analysis_tokens"] = analysis_tokens
        result["analysis_cost"] = analysis_cost
        print(
            f"  Analysis: {analysis_time:.1f}s, {analysis_tokens} tokens, ${analysis_cost:.4f}"
        )

        # Step 2: Translation (if needed)
        if translate and source_lang:
            print("  Translating to Russian...")
            trans_prompt = translation_prompt(analysis, source_lang)
            translation, trans_time, trans_tokens, trans_cost = await call_api(
                client, api_key, GEMINI_TRANSLATE, trans_prompt
            )

            result["translation_time"] = trans_time
            result["translation_tokens"] = trans_tokens
            result["translation_cost"] = trans_cost
            result["response"] = translation
            result["original"] = analysis

            result["total_time"] = analysis_time + trans_time
            result["total_tokens"] = analysis_tokens + trans_tokens
            result["total_cost"] = analysis_cost + trans_cost

            print(
                f"  Translation: {trans_time:.1f}s, {trans_tokens} tokens, ${trans_cost:.4f}"
            )
        else:
            result["response"] = analysis
            result["total_time"] = analysis_time
            result["total_tokens"] = analysis_tokens
            result["total_cost"] = analysis_cost

        print(
            f"  TOTAL: {result['total_time']:.1f}s, {result['total_tokens']} tokens, ${result['total_cost']:.4f}"
        )
        result["success"] = True

    except Exception as e:
        print(f"  ERROR: {e}")
        result["error"] = str(e)
        result["success"] = False

    return result


async def main():
    api_key = os.environ.get("OPENROUTER_API_KEY")
    if not api_key:
        print("Error: OPENROUTER_API_KEY not set")
        return

    scores = format_scores(PROFILE)

    # Define pipelines
    pipelines = [
        {
            "name": "1. Opus RU (direct)",
            "model": OPUS,
            "prompt": prompt_russian(scores, USER_CONTEXT),
            "translate": False,
        },
        {
            "name": "2. Opus EN + translate",
            "model": OPUS,
            "prompt": prompt_english(scores, USER_CONTEXT),
            "translate": True,
            "source_lang": "en",
        },
        {
            "name": "3. Qwen RU (direct)",
            "model": QWEN,
            "prompt": prompt_russian(scores, USER_CONTEXT),
            "translate": False,
        },
        {
            "name": "4. Qwen ZH + translate",
            "model": QWEN,
            "prompt": prompt_chinese(scores, USER_CONTEXT),
            "translate": True,
            "source_lang": "zh",
        },
        {
            "name": "5. DeepSeek RU (direct)",
            "model": DEEPSEEK,
            "prompt": prompt_russian(scores, USER_CONTEXT),
            "translate": False,
        },
        {
            "name": "6. DeepSeek ZH + translate",
            "model": DEEPSEEK,
            "prompt": prompt_chinese(scores, USER_CONTEXT),
            "translate": True,
            "source_lang": "zh",
        },
    ]

    print(f"\n{'#' * 60}")
    print(f"# Pipeline Comparison Test")
    print(f"# {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"# Context: {USER_CONTEXT}")
    print(f"{'#' * 60}")

    results = []

    async with httpx.AsyncClient() as client:
        for p in pipelines:
            result = await run_pipeline(
                client,
                api_key,
                p["name"],
                p["model"],
                p["prompt"],
                p.get("translate", False),
                p.get("source_lang"),
            )
            results.append(result)

    # Summary
    print(f"\n{'=' * 60}")
    print("SUMMARY")
    print(f"{'=' * 60}")
    print(f"{'Pipeline':<30} {'Time':>8} {'Tokens':>8} {'Cost':>10} {'Status':>8}")
    print("-" * 70)

    for r in results:
        if r["success"]:
            print(
                f"{r['name']:<30} {r['total_time']:>7.1f}s {r['total_tokens']:>8} ${r['total_cost']:>9.4f} {'OK':>8}"
            )
        else:
            print(f"{r['name']:<30} {'':>8} {'':>8} {'':>10} {'FAILED':>8}")

    # Save results to separate MD files
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_dir = f"scripts/pipeline_tests/{timestamp}"
    os.makedirs(output_dir, exist_ok=True)

    # Save each result to separate file
    for i, r in enumerate(results, 1):
        filename = f"{output_dir}/{i:02d}_{r['name'].split('.')[1].strip().replace(' ', '_').replace('+', '_')}.md"

        with open(filename, "w") as f:
            f.write(f"# {r['name']}\n\n")
            if r["success"]:
                f.write(f"- **Model:** {r['model']}\n")
                f.write(f"- **Total Time:** {r['total_time']:.1f}s\n")
                f.write(f"- **Total Tokens:** {r['total_tokens']}\n")
                f.write(f"- **Total Cost:** ${r['total_cost']:.4f}\n")

                if r.get("translate"):
                    f.write(f"- **Analysis Time:** {r['analysis_time']:.1f}s\n")
                    f.write(f"- **Translation Time:** {r['translation_time']:.1f}s\n")

                f.write("\n## Response\n\n")
                f.write(r["response"])

                if r.get("original"):
                    f.write("\n\n---\n\n## Original (before translation)\n\n")
                    f.write(r["original"])
            else:
                f.write(f"**Error:** {r.get('error', 'Unknown')}\n")

        print(f"Saved: {filename}")

    # Create combined file
    combined_file = f"{output_dir}/00_COMBINED.md"

    with open(combined_file, "w") as f:
        f.write("# Pipeline Comparison Results\n\n")
        f.write(f"**Date:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write(f"**Context:** {USER_CONTEXT}\n\n")

        f.write("## Summary\n\n")
        f.write("| Pipeline | Time | Tokens | Cost | Status |\n")
        f.write("|----------|------|--------|------|--------|\n")
        for r in results:
            if r["success"]:
                f.write(
                    f"| {r['name']} | {r['total_time']:.1f}s | {r['total_tokens']} | ${r['total_cost']:.4f} | OK |\n"
                )
            else:
                f.write(f"| {r['name']} | - | - | - | FAILED |\n")

        f.write("\n---\n\n")

        # Append each individual file
        for i, r in enumerate(results, 1):
            filename = f"{output_dir}/{i:02d}_{r['name'].split('.')[1].strip().replace(' ', '_').replace('+', '_')}.md"
            with open(filename, "r") as rf:
                f.write(rf.read())
            f.write("\n\n---\n\n")

    print(f"\nCombined results saved to: {combined_file}")


if __name__ == "__main__":
    asyncio.run(main())
