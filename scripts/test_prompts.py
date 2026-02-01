#!/usr/bin/env python3
"""Test and compare different prompt versions for personality analysis."""

import asyncio
import os
import time
from datetime import datetime

import httpx

MODEL = "qwen/qwen3-235b-a22b-2507"

# Test profile data
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


# Current prompt (baseline)
def prompt_v1(scores: str, context: str) -> str:
    context_section = (
        f"""
# Контекст пользователя

Человек рассказал о себе следующее:
"{context}"

Используй этот контекст, чтобы сделать анализ более персонализированным и релевантным.
"""
        if context
        else ""
    )

    return f"""Ты эксперт по психологии личности, специализирующийся на модели Big Five. Проанализируй результаты теста IPIP-NEO-120 и напиши подробный психологический портрет.

# Результаты теста (IPIP-NEO-120)

Диапазон баллов: домены 24-120 (5 доменов x 24 вопроса), фасеты 4-20 (6 фасетов на домен x 4 вопроса).
- Низкий: ниже 40%
- Средний: 40-60%
- Высокий: выше 60%
{scores}
{context_section}
# Структура ответа

## 1. Обзор (1 абзац)
Краткий синтез профиля личности - что делает этого человека уникальным, ключевые контрасты и паттерны.

## 2. Анализ по доменам (5 разделов)
Для каждого из пяти доменов Big Five:
- Общая интерпретация балла домена
- Заметные паттерны фасетов (особенно экстремальные значения и интересные контрасты)
- Как это проявляется в поведении и жизни
- Взаимодействия с другими доменами

## 3. Сильные стороны (список)
5-7 ключевых сильных сторон на основе профиля. Конкретные, не общие.

## 4. Зоны роста (список)
3-5 потенциальных вызовов или областей для развития. Конструктивная формулировка.

## 5. Синтез (1 абзац)
Интегративное резюме: какой тип личности вырисовывается из данных, как разные черты взаимодействуют.

# Стилевые указания

1. Пиши на русском, используй неформальное "ты".
2. Будь конкретен - указывай реальные проценты и названия фасетов
3. Ищи паттерны: контрасты внутри доменов, взаимодействия между доменами
4. Избегай общих утверждений, которые могут подойти кому угодно
5. Используй конкретные поведенческие примеры где возможно
6. Будь сбалансирован - признавай сложность, избегай упрощений

Начни анализ:"""


# New prompt v2 - more concise, with recommendations
def prompt_v2(scores: str, context: str) -> str:
    context_section = (
        f"""
## О человеке
{context}
"""
        if context
        else ""
    )

    return f"""Проанализируй результаты теста Big Five (IPIP-NEO-120) и напиши персонализированный психологический портрет.

# Данные теста
Домены: 24-120 баллов. Фасеты: 4-20 баллов.
Низкий <40%, Средний 40-60%, Высокий >60%
{scores}
{context_section}
# Формат ответа

## Обзор
1 абзац: уникальный профиль личности, ключевые паттерны и контрасты.

## Нейротизм / Экстраверсия / Открытость / Доброжелательность / Добросовестность
По каждому домену: интерпретация балла, интересные фасетные паттерны, проявление в жизни.

## Сильные стороны
5-6 конкретных преимуществ из профиля (не общие фразы).

## Зоны развития  
3-4 реальных вызова/слабости (честно, но конструктивно).

## Рекомендации
4-5 практических советов для личностного роста, учитывающих:
- Специфику профиля (сильные стороны как ресурс)
- Контекст жизни человека
- Конкретные действия, а не абстрактные пожелания

## Заключение
Интегративное резюме: тип личности, взаимодействие черт.

# Стиль
- Русский язык, обращение на "ты"
- Конкретика: проценты, названия фасетов, примеры поведения
- Честность без жёсткости
- Персонализация под контекст"""


# Prompt v3 - even more streamlined
def prompt_v3(scores: str, context: str) -> str:
    return f"""Напиши психологический портрет по результатам Big Five теста.

# Результаты (IPIP-NEO-120)
Шкала: домены 24-120, фасеты 4-20. Низкий <40%, средний 40-60%, высокий >60%.
{scores}
{"## Контекст: " + context if context else ""}

# Структура анализа

**Обзор** — 1 абзац об уникальности профиля и ключевых паттернах.

**Анализ доменов** — для каждого из 5 доменов: общий балл, важные фасеты, как проявляется в жизни.

**Сильные стороны** — 5-6 конкретных преимуществ (с привязкой к фасетам).

**Слабые стороны** — 3-4 реальных проблемных зоны (честно, но не жёстко).

**Рекомендации** — 4-5 практических действий:
• Что делать, чтобы использовать сильные стороны
• Как компенсировать слабости
• Привязка к контексту жизни человека

**Итог** — какой тип личности, как черты взаимодействуют.

Пиши на русском, на "ты". Ссылайся на конкретные баллы и фасеты. Избегай банальностей."""


# Prompt v4 - minimal but effective
def prompt_v4(scores: str, context: str) -> str:
    return f"""Напиши психологический портрет по Big Five (IPIP-NEO-120).

# Данные
Домены: 24-120 (низкий <40%, средний 40-60%, высокий >60%). Фасеты: 4-20.
{scores}
{"**Контекст:** " + context if context else ""}

# Ответ (markdown)

## Обзор
Краткая характеристика уникальности профиля и главных паттернов.

## Нейротизм | Экстраверсия | Открытость | Доброжелательность | Сознательность
Каждый домен отдельной секцией: что означает балл, ключевые фасеты, проявление в жизни.

## Сильные стороны
5-6 пунктов с привязкой к фасетам. Конкретика, не клише.

## Слабые стороны
3-4 честных проблемных зоны. Не сглаживать, но без жёсткости.

## Рекомендации
5-6 практических советов:
- Опираться на сильные стороны как ресурс
- Компенсировать слабости конкретными действиями
- Учитывать контекст жизни человека
- Давать чёткие, выполнимые шаги

## Итог
Какой тип личности, как черты взаимодействуют.

---
Стиль: русский, на "ты", ссылки на % и фасеты, без воды."""


# Prompt v5 - even shorter, trust the model more
def prompt_v5(scores: str, context: str) -> str:
    return f"""Big Five анализ (IPIP-NEO-120). Домены 24-120, фасеты 4-20. <40% низкий, 40-60% средний, >60% высокий.

{scores}
{("Контекст: " + context) if context else ""}

Напиши психологический портрет:

1. **Обзор** — уникальность профиля, ключевые паттерны
2. **5 доменов** — интерпретация, важные фасеты, проявления в жизни
3. **Сильные стороны** — 5-6 конкретных преимуществ (с фасетами)
4. **Слабые стороны** — 3-4 реальных проблемы (честно)
5. **Рекомендации** — 5-6 практических действий (опора на сильные, работа со слабыми, привязка к контексту)
6. **Итог** — тип личности, взаимодействие черт

Русский, на "ты", конкретика (проценты, фасеты), без банальностей."""


# Prompt v6 - final optimized version
def prompt_v6(scores: str, context: str) -> str:
    return f"""Big Five (IPIP-NEO-120). Домены 24-120, фасеты 4-20. Низкий <40%, средний 40-60%, высокий >60%.

{scores}
{("**О человеке:** " + context) if context else ""}

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


PROMPTS = {
    "v1_current": prompt_v1,
    "v2_with_recs": prompt_v2,
    "v3_streamlined": prompt_v3,
    "v4_minimal": prompt_v4,
    "v5_shortest": prompt_v5,
    "v6_final": prompt_v6,
}


async def test_prompt(
    client: httpx.AsyncClient,
    api_key: str,
    name: str,
    prompt: str,
) -> tuple[str, str, float, int]:
    """Test a prompt. Returns (name, response, time, tokens)."""
    print(f"\n{'=' * 60}")
    print(f"Testing: {name}")
    print(f"Prompt length: {len(prompt)} chars")
    print(f"{'=' * 60}")

    start = time.time()

    try:
        resp = await client.post(
            "https://openrouter.ai/api/v1/chat/completions",
            headers={
                "Authorization": f"Bearer {api_key}",
                "Content-Type": "application/json",
            },
            json={
                "model": MODEL,
                "max_tokens": 8192,
                "messages": [{"role": "user", "content": prompt}],
            },
            timeout=300.0,
        )

        elapsed = time.time() - start

        if resp.status_code != 200:
            print(f"ERROR: HTTP {resp.status_code}")
            return name, f"ERROR: {resp.text[:200]}", elapsed, 0

        data = resp.json()
        content = data["choices"][0]["message"]["content"]
        tokens = data.get("usage", {}).get("completion_tokens", 0)

        print(f"Time: {elapsed:.1f}s, Tokens: {tokens}")
        print(f"Response length: {len(content)} chars")

        return name, content, elapsed, tokens

    except Exception as e:
        return name, f"ERROR: {e}", time.time() - start, 0


async def main():
    api_key = os.environ.get("OPENROUTER_API_KEY")
    if not api_key:
        print("Error: OPENROUTER_API_KEY not set")
        return

    print(f"Model: {MODEL}")
    print(f"Context: {USER_CONTEXT}")

    scores = format_scores(PROFILE)

    # Test only one prompt at a time to compare
    prompt_name = os.environ.get("PROMPT_VERSION", "v3_streamlined")

    if prompt_name not in PROMPTS:
        print(f"Available prompts: {list(PROMPTS.keys())}")
        return

    prompt_fn = PROMPTS[prompt_name]
    prompt = prompt_fn(scores, USER_CONTEXT)

    async with httpx.AsyncClient() as client:
        name, response, elapsed, tokens = await test_prompt(
            client, api_key, prompt_name, prompt
        )

    # Save result
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = f"scripts/prompt_tests/{prompt_name}_{timestamp}.md"
    os.makedirs("scripts/prompt_tests", exist_ok=True)

    with open(output_file, "w") as f:
        f.write(f"# Prompt Test: {prompt_name}\n\n")
        f.write(f"**Model:** {MODEL}\n")
        f.write(f"**Time:** {elapsed:.1f}s\n")
        f.write(f"**Tokens:** {tokens}\n")
        f.write(f"**Context:** {USER_CONTEXT}\n\n")
        f.write("---\n\n")
        f.write("## Prompt\n\n```\n")
        f.write(prompt)
        f.write("\n```\n\n---\n\n")
        f.write("## Response\n\n")
        f.write(response)

    print(f"\nSaved to: {output_file}")
    print("\n" + "=" * 60)
    print("RESPONSE:")
    print("=" * 60)
    print(response)


if __name__ == "__main__":
    asyncio.run(main())
