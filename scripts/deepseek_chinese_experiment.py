#!/usr/bin/env python3
"""Experiment: DeepSeek V3.2 writes in Chinese, Gemini Flash Lite translates to Russian."""

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

用中文撰写，使用非正式的"你"。具体引用百分比和方面名称。避免泛泛之谈。深入分析，提供有价值的洞察。"""


def build_prompt_russian() -> str:
    """Direct Russian prompt for comparison."""
    scores = build_scores_text(PROFILE)
    return f"""Ты эксперт по психологии личности, специализирующийся на модели Big Five. Проанализируй результаты теста IPIP-NEO-120 и напиши подробный психологический портрет.

# Результаты теста (IPIP-NEO-120)
Диапазон баллов: домены 24-120, фасеты 4-20. Низкий: <40%, Средний: 40-60%, Высокий: >60%
{scores}

# Контекст
"Данила, 24 года, middle Rust разработчик на удалёнке, проблемы с организованностью, планированием, распорядком сна."

# Структура ответа
## 1. Обзор (1 абзац) - что делает этого человека уникальным, ключевые контрасты
## 2. Анализ по доменам (5 разделов) - интерпретация, паттерны фасетов, поведение, взаимодействия
## 3. Сильные стороны (5-7 пунктов) - конкретные, не общие
## 4. Зоны роста (3-5 пунктов) - конструктивная формулировка
## 5. Синтез (1 абзац) - интегративное резюме

Пиши на русском, используй неформальное "ты". Указывай конкретные проценты. Избегай общих фраз."""


def build_translation_prompt(text: str) -> str:
    return f"""Переведи этот психологический анализ личности с китайского на русский язык.

Требования:
1. Используй неформальное обращение на "ты"
2. Психологические термины должны звучать естественно по-русски
3. Сохрани оригинальные названия: IPIP-NEO-120, Big Five, Rust, Данила
4. Сохрани форматирование markdown
5. Перевод должен звучать естественно, не дословно
6. Сохрани все детали и нюансы оригинала

Текст для перевода:

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
            timeout=600.0,  # DeepSeek can be slow
        )
        elapsed = time.time() - start
        if resp.status_code != 200:
            return {"error": f"HTTP {resp.status_code}: {resp.text}", "time": elapsed}
        data = resp.json()
        return {
            "content": data["choices"][0]["message"]["content"],
            "time": elapsed,
            "tokens": data.get("usage", {}),
        }
    except Exception as e:
        return {"error": str(e), "time": time.time() - start}


async def main():
    api_key = os.environ.get("OPENROUTER_API_KEY")
    if not api_key:
        print("Error: OPENROUTER_API_KEY not set")
        return

    log("=" * 70)
    log("DeepSeek Chinese Experiment: DeepSeek V3.2 (ZH) -> Gemini Flash Lite (RU)")
    log("=" * 70)

    deepseek_model = "deepseek/deepseek-chat-v3-0324"  # DeepSeek V3.2
    translator_model = "google/gemini-2.5-flash-lite"

    results = {}

    async with httpx.AsyncClient() as client:
        # Test 1: DeepSeek in Chinese -> translate to Russian
        log("\n[1/2] DeepSeek V3.2 in CHINESE...")
        zh_result = await query(client, deepseek_model, build_prompt_chinese(), api_key)

        if "error" in zh_result:
            log(f"  ERROR: {zh_result['error']}")
            return

        tokens_info = zh_result.get("tokens", {})
        log(f"  Analysis: {zh_result['time']:.1f}s")
        log(
            f"  Tokens: {tokens_info.get('prompt_tokens', '?')} in, {tokens_info.get('completion_tokens', '?')} out"
        )
        log(f"  Output length: {len(zh_result['content'])} chars")

        log("\n  Translating to Russian with Gemini Flash Lite...")
        translation = await query(
            client,
            translator_model,
            build_translation_prompt(zh_result["content"]),
            api_key,
        )

        if "error" in translation:
            log(f"  Translation ERROR: {translation['error']}")
        else:
            trans_tokens = translation.get("tokens", {})
            log(f"  Translation: {translation['time']:.1f}s")
            log(
                f"  Tokens: {trans_tokens.get('prompt_tokens', '?')} in, {trans_tokens.get('completion_tokens', '?')} out"
            )

            total_time = zh_result["time"] + translation["time"]
            results["deepseek_zh_translated"] = {
                "original": zh_result,
                "translation": translation,
                "total_time": total_time,
            }

        # Test 2: DeepSeek direct in Russian for comparison
        log("\n[2/2] DeepSeek V3.2 DIRECT in Russian (for comparison)...")
        ru_result = await query(client, deepseek_model, build_prompt_russian(), api_key)

        if "error" in ru_result:
            log(f"  ERROR: {ru_result['error']}")
        else:
            ru_tokens = ru_result.get("tokens", {})
            log(f"  Analysis: {ru_result['time']:.1f}s")
            log(
                f"  Tokens: {ru_tokens.get('prompt_tokens', '?')} in, {ru_tokens.get('completion_tokens', '?')} out"
            )
            log(f"  Output length: {len(ru_result['content'])} chars")
            results["deepseek_ru_direct"] = ru_result

    # Save results
    log("\n" + "=" * 70)
    log("RESULTS SUMMARY")
    log("=" * 70)

    output_dir = "model_outputs"
    os.makedirs(output_dir, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

    if "deepseek_zh_translated" in results:
        data = results["deepseek_zh_translated"]
        log(f"\nDeepSeek (ZH) + Gemini translation:")
        log(f"  Analysis time: {data['original']['time']:.1f}s")
        log(f"  Translation time: {data['translation']['time']:.1f}s")
        log(f"  Total time: {data['total_time']:.1f}s")

        # Save original Chinese
        with open(f"{output_dir}/deepseek_chinese_{timestamp}.md", "w") as f:
            f.write("# DeepSeek V3.2 Analysis (Chinese)\n\n")
            f.write(f"**Time:** {data['original']['time']:.1f}s\n")
            f.write(f"**Tokens:** {data['original'].get('tokens', {})}\n\n---\n\n")
            f.write(data["original"]["content"])
        log(f"  Saved: deepseek_chinese_{timestamp}.md")

        # Save Russian translation
        with open(f"{output_dir}/deepseek_zh_to_ru_{timestamp}.md", "w") as f:
            f.write("# DeepSeek V3.2 (Chinese) -> Russian Translation\n\n")
            f.write(f"**Analysis time:** {data['original']['time']:.1f}s\n")
            f.write(f"**Translation time:** {data['translation']['time']:.1f}s\n")
            f.write(f"**Total time:** {data['total_time']:.1f}s\n\n---\n\n")
            f.write(data["translation"]["content"])
        log(f"  Saved: deepseek_zh_to_ru_{timestamp}.md")

    if "deepseek_ru_direct" in results:
        data = results["deepseek_ru_direct"]
        log(f"\nDeepSeek (RU) direct:")
        log(f"  Time: {data['time']:.1f}s")

        with open(f"{output_dir}/deepseek_russian_direct_{timestamp}.md", "w") as f:
            f.write("# DeepSeek V3.2 Analysis (Direct Russian)\n\n")
            f.write(f"**Time:** {data['time']:.1f}s\n")
            f.write(f"**Tokens:** {data.get('tokens', {})}\n\n---\n\n")
            f.write(data["content"])
        log(f"  Saved: deepseek_russian_direct_{timestamp}.md")

    # Summary comparison
    log("\n" + "-" * 50)
    log("COMPARISON:")
    if "deepseek_zh_translated" in results:
        log(
            f"  DeepSeek (ZH) + translate: {results['deepseek_zh_translated']['total_time']:.1f}s"
        )
    if "deepseek_ru_direct" in results:
        log(
            f"  DeepSeek (RU) direct:      {results['deepseek_ru_direct']['time']:.1f}s"
        )
    log("\nPrevious best results for reference:")
    log("  Qwen (RU) direct: 33.5s, quality 9/10")
    log("  DeepSeek V3.2 (RU): 214s (was slow before)")


if __name__ == "__main__":
    asyncio.run(main())
