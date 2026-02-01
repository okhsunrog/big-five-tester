#!/usr/bin/env python3
"""Experiment: Compare Qwen in English, Chinese, then translate to Russian."""

import asyncio
import os
import time
from datetime import datetime

import httpx

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


def build_scores_text(profile: dict) -> str:
    scores = ""
    for domain in profile["domains"]:
        domain_pct = (domain["raw"] / 120) * 100
        scores += f"\n## {domain['domain']} ({domain['raw']}/120, {domain_pct:.0f}%)\n"
        for facet in domain["facets"]:
            facet_pct = (facet["raw"] / 20) * 100
            scores += f"- {facet['facet']}: {facet['raw']}/20 ({facet_pct:.0f}%)\n"
    return scores


def build_prompt_english() -> str:
    scores = build_scores_text(PROFILE)
    return f"""You are a personality psychology expert specializing in the Big Five model. Analyze the IPIP-NEO-120 test results and write a comprehensive personality profile.

# Test Results (IPIP-NEO-120)
Scores: 24-120 for domains, 4-20 for facets. Low: <40%, Neutral: 40-60%, High: >60%
{scores}

# User Context
"Danila, 24 years old, middle Rust developer working remotely, struggles with organization, planning, and sleep schedule."

# Output Structure
## 1. Overview (1 paragraph) - what makes this person unique, key contrasts
## 2. Analysis by Domain (5 sections) - interpretation, facet patterns, behavior, interactions
## 3. Strengths (5-7 bullet points) - specific, not generic
## 4. Growth Areas (3-5 bullet points) - constructive framing
## 5. Synthesis (1 paragraph) - integrative summary

Write in English, use informal "you". Be specific with percentages. Avoid generic statements."""


def build_prompt_chinese() -> str:
    scores = build_scores_text(PROFILE)
    return f"""你是专门研究大五人格模型的人格心理学专家。请分析以下IPIP-NEO-120测试结果，撰写一份全面深入的人格分析报告。

# 测试结果 (IPIP-NEO-120)
分数范围：领域24-120分，方面4-20分。低：<40%，中：40-60%，高：>60%
{scores}

# 用户背景
"Данила（达尼拉），24岁，中级Rust开发者，远程工作，在组织能力、计划安排和睡眠作息方面存在困难。"

# 输出结构
## 1. 概述（1段）- 这个人的独特之处，关键对比和模式
## 2. 各维度分析（5个部分）- 每个大五维度的解读、显著的方面模式、行为表现、与其他维度的交互
## 3. 优势（5-7个要点）- 具体而非笼统
## 4. 成长领域（3-5个要点）- 以建设性的方式表述
## 5. 综合（1段）- 整合性总结，描述这些特质如何相互作用

用中文撰写，使用非正式的"你"。具体引用百分比和方面名称。避免泛泛之谈。"""


def build_translation_prompt(text: str, source_lang: str) -> str:
    return f"""Translate this personality analysis from {source_lang} to Russian.

Requirements:
1. Use informal "ты" form
2. Keep psychological terms accurate but natural in Russian
3. Keep proper names/acronyms as-is: IPIP-NEO-120, Big Five, Rust, Данила
4. Preserve markdown formatting
5. Make it sound natural, not like a literal translation

Text to translate:

{text}"""


def log(msg: str):
    print(f"[{datetime.now().strftime('%H:%M:%S')}] {msg}", flush=True)


async def query(
    client: httpx.AsyncClient, model: str, prompt: str, api_key: str
) -> dict:
    start = time.time()
    try:
        resp = await client.post(
            "https://openrouter.ai/api/v1/chat/completions",
            headers={
                "Authorization": f"Bearer {api_key}",
                "Content-Type": "application/json",
            },
            json={
                "model": model,
                "max_tokens": 8192,
                "messages": [{"role": "user", "content": prompt}],
            },
            timeout=300.0,
        )
        elapsed = time.time() - start
        if resp.status_code != 200:
            return {"error": f"HTTP {resp.status_code}", "time": elapsed}
        data = resp.json()
        return {
            "content": data["choices"][0]["message"]["content"],
            "time": elapsed,
            "tokens": data.get("usage", {}).get("completion_tokens", 0),
        }
    except Exception as e:
        return {"error": str(e), "time": time.time() - start}


async def main():
    api_key = os.environ.get("OPENROUTER_API_KEY")
    if not api_key:
        print("Error: OPENROUTER_API_KEY not set")
        return

    log("=" * 60)
    log("Qwen Language Experiment: EN vs ZH -> Russian translation")
    log("=" * 60)

    qwen_model = "qwen/qwen3-235b-a22b-2507"
    translator = "google/gemini-2.5-flash-lite"

    results = {}

    async with httpx.AsyncClient() as client:
        # Test 1: Qwen English -> translate
        log("\n[1/2] Qwen in ENGLISH...")
        en_result = await query(client, qwen_model, build_prompt_english(), api_key)
        if "error" in en_result:
            log(f"  ERROR: {en_result['error']}")
        else:
            log(
                f"  Analysis: {en_result['time']:.1f}s, {en_result['tokens']} tokens, {len(en_result['content'])} chars"
            )

            log("  Translating to Russian...")
            en_trans = await query(
                client,
                translator,
                build_translation_prompt(en_result["content"], "English"),
                api_key,
            )
            if "error" not in en_trans:
                log(
                    f"  Translation: {en_trans['time']:.1f}s, {en_trans['tokens']} tokens"
                )
                results["english"] = {
                    "original": en_result,
                    "translation": en_trans,
                    "total_time": en_result["time"] + en_trans["time"],
                }

        # Test 2: Qwen Chinese -> translate
        log("\n[2/2] Qwen in CHINESE...")
        zh_result = await query(client, qwen_model, build_prompt_chinese(), api_key)
        if "error" in zh_result:
            log(f"  ERROR: {zh_result['error']}")
        else:
            log(
                f"  Analysis: {zh_result['time']:.1f}s, {zh_result['tokens']} tokens, {len(zh_result['content'])} chars"
            )

            log("  Translating to Russian...")
            zh_trans = await query(
                client,
                translator,
                build_translation_prompt(zh_result["content"], "Chinese"),
                api_key,
            )
            if "error" not in zh_trans:
                log(
                    f"  Translation: {zh_trans['time']:.1f}s, {zh_trans['tokens']} tokens"
                )
                results["chinese"] = {
                    "original": zh_result,
                    "translation": zh_trans,
                    "total_time": zh_result["time"] + zh_trans["time"],
                }

    # Save results
    log("\n" + "=" * 60)
    log("RESULTS SUMMARY")
    log("=" * 60)

    output_dir = "model_outputs"
    os.makedirs(output_dir, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

    for lang, data in results.items():
        log(f"\n{lang.upper()}:")
        log(f"  Analysis time: {data['original']['time']:.1f}s")
        log(f"  Translation time: {data['translation']['time']:.1f}s")
        log(f"  Total time: {data['total_time']:.1f}s")

        # Save original
        with open(f"{output_dir}/qwen_{lang}_{timestamp}_original.md", "w") as f:
            f.write(f"# Qwen Analysis ({lang.upper()})\n\n")
            f.write(
                f"**Time:** {data['original']['time']:.1f}s | **Tokens:** {data['original']['tokens']}\n\n---\n\n"
            )
            f.write(data["original"]["content"])

        # Save translation
        with open(f"{output_dir}/qwen_{lang}_{timestamp}_translated.md", "w") as f:
            f.write(f"# Qwen ({lang.upper()}) -> Russian Translation\n\n")
            f.write(f"**Analysis time:** {data['original']['time']:.1f}s\n")
            f.write(f"**Translation time:** {data['translation']['time']:.1f}s\n")
            f.write(f"**Total time:** {data['total_time']:.1f}s\n\n---\n\n")
            f.write(data["translation"]["content"])

        log(f"  Saved to: qwen_{lang}_{timestamp}_*.md")

    # Comparison with direct Russian (from previous test)
    log("\n" + "-" * 40)
    log("COMPARISON (vs direct Russian from previous test):")
    log("  Qwen (RU) direct: 33.5s, quality 9/10")
    for lang, data in results.items():
        log(f"  Qwen ({lang.upper()}) + translate: {data['total_time']:.1f}s")


if __name__ == "__main__":
    asyncio.run(main())
