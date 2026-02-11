//! Simple manual i18n implementation.
//!
//! Supports English (en) and Russian (ru) locales with URL-based routing.

use leptos::prelude::*;
use leptos_router::hooks::use_location;
use serde::{Deserialize, Serialize};

/// Supported locales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Locale {
    #[default]
    En,
    Ru,
}

impl Locale {
    /// Get locale from URL path segment.
    pub fn from_path(path: &str) -> Self {
        let first_segment = path.trim_start_matches('/').split('/').next().unwrap_or("");
        match first_segment {
            "ru" => Locale::Ru,
            _ => Locale::En,
        }
    }

    /// Get the URL path prefix for this locale.
    pub fn path_prefix(&self) -> &'static str {
        match self {
            Locale::En => "/en",
            Locale::Ru => "/ru",
        }
    }

    /// Get language code.
    pub fn code(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Ru => "ru",
        }
    }
}

/// I18n context holding the current locale.
#[derive(Clone, Copy)]
pub struct I18nContext {
    locale: RwSignal<Locale>,
}

impl I18nContext {
    /// Get the current locale.
    pub fn get_locale(&self) -> Locale {
        self.locale.get()
    }

    /// Set the locale (navigates to the new locale path).
    pub fn set_locale(&self, locale: Locale) {
        self.locale.set(locale);

        // Navigate to the new locale path
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let current_path = window.location().pathname().unwrap_or_default();
                let new_path = switch_locale_in_path(&current_path, locale);
                let _ = window.location().set_pathname(&new_path);
            }
        }
    }
}

/// Switch the locale in a URL path.
#[cfg(target_arch = "wasm32")]
fn switch_locale_in_path(path: &str, new_locale: Locale) -> String {
    let path = path.trim_start_matches('/');
    let segments: Vec<&str> = path.split('/').collect();

    let rest = if !segments.is_empty() && (segments[0] == "en" || segments[0] == "ru") {
        segments[1..].join("/")
    } else {
        segments.join("/")
    };

    format!("{}/{}", new_locale.path_prefix(), rest)
}

/// Provide i18n context to the application.
#[component]
pub fn I18nProvider(children: Children) -> impl IntoView {
    let location = use_location();

    // Initialize with correct locale from URL immediately (prevents blinking on load)
    let initial_locale = Locale::from_path(&location.pathname.get_untracked());
    let locale = RwSignal::new(initial_locale);

    // Update locale when URL changes (for navigation)
    Effect::new(move |_| {
        let path = location.pathname.get();
        let detected = Locale::from_path(&path);
        // Only update if locale actually changed to prevent unnecessary re-renders
        if detected != locale.get_untracked() {
            locale.set(detected);
        }
    });

    let ctx = I18nContext { locale };
    provide_context(ctx);

    children()
}

/// Get the i18n context.
pub fn use_i18n() -> I18nContext {
    expect_context::<I18nContext>()
}

// ============================================================================
// Translations
// ============================================================================

/// Get a translation string.
pub fn t(locale: Locale, key: &str) -> &'static str {
    match (locale, key) {
        // Title
        (Locale::En, "title") => "Big Five Personality Test",
        (Locale::Ru, "title") => "Тест личности Big Five",

        // Home page
        (Locale::En, "home_subtitle") => "Discover Your Personality",
        (Locale::Ru, "home_subtitle") => "Узнайте свою личность",

        (Locale::En, "home_description") => {
            "The Big Five personality test is a scientific assessment that measures five key dimensions of your personality. This version uses the IPIP-NEO-120 inventory, a well-validated instrument based on decades of psychological research."
        }
        (Locale::Ru, "home_description") => {
            "Тест личности Big Five - это научная оценка, измеряющая пять ключевых измерений вашей личности. Эта версия использует опросник IPIP-NEO-120, хорошо валидированный инструмент, основанный на десятилетиях психологических исследований."
        }

        (Locale::En, "home_what_measured") => "What This Test Measures",
        (Locale::Ru, "home_what_measured") => "Что измеряет этот тест",

        (Locale::En, "home_questions_count") => "120 questions",
        (Locale::Ru, "home_questions_count") => "120 вопросов",

        (Locale::En, "home_time_estimate") => "~15 minutes",
        (Locale::Ru, "home_time_estimate") => "~15 минут",

        (Locale::En, "home_start_button") => "Start Test",
        (Locale::Ru, "home_start_button") => "Начать тест",

        // Domain descriptions
        (Locale::En, "domain_n_desc") => "Emotional sensitivity and stress response",
        (Locale::Ru, "domain_n_desc") => "Эмоциональная чувствительность и реакция на стресс",

        (Locale::En, "domain_e_desc") => "Social engagement and energy levels",
        (Locale::Ru, "domain_e_desc") => "Социальная активность и уровень энергии",

        (Locale::En, "domain_o_desc") => "Creativity and intellectual curiosity",
        (Locale::Ru, "domain_o_desc") => "Креативность и интеллектуальное любопытство",

        (Locale::En, "domain_a_desc") => "Cooperation and empathy towards others",
        (Locale::Ru, "domain_a_desc") => "Сотрудничество и эмпатия к другим",

        (Locale::En, "domain_c_desc") => "Organization and goal-directed behavior",
        (Locale::Ru, "domain_c_desc") => "Организованность и целенаправленное поведение",

        // Test page
        (Locale::En, "test_question") => "Question",
        (Locale::Ru, "test_question") => "Вопрос",

        (Locale::En, "test_back") => "Back",
        (Locale::Ru, "test_back") => "Назад",

        (Locale::En, "test_next") => "Next",
        (Locale::Ru, "test_next") => "Далее",

        (Locale::En, "test_show_results") => "Show Results",
        (Locale::Ru, "test_show_results") => "Показать результаты",

        (Locale::En, "test_answered") => "Answered",
        (Locale::Ru, "test_answered") => "Отвечено",

        // Answer options
        (Locale::En, "answer_1") => "Very Inaccurate",
        (Locale::Ru, "answer_1") => "Совершенно не соответствует",

        (Locale::En, "answer_2") => "Moderately Inaccurate",
        (Locale::Ru, "answer_2") => "Скорее не соответствует",

        (Locale::En, "answer_3") => "Neither Accurate Nor Inaccurate",
        (Locale::Ru, "answer_3") => "Нейтрально",

        (Locale::En, "answer_4") => "Moderately Accurate",
        (Locale::Ru, "answer_4") => "Скорее соответствует",

        (Locale::En, "answer_5") => "Very Accurate",
        (Locale::Ru, "answer_5") => "Полностью соответствует",

        // Results page
        (Locale::En, "results_title") => "Your Results",
        (Locale::Ru, "results_title") => "Ваши результаты",

        (Locale::En, "results_ai_title") => "AI Personality Analysis",
        (Locale::Ru, "results_ai_title") => "AI-анализ личности",

        (Locale::En, "results_ai_description") => {
            "Get a personalized AI-generated description of your personality based on your test results."
        }
        (Locale::Ru, "results_ai_description") => {
            "Получите персонализированное AI-описание вашей личности на основе результатов теста."
        }

        (Locale::En, "results_ai_button") => "Generate AI Analysis",
        (Locale::Ru, "results_ai_button") => "Сгенерировать AI-анализ",

        (Locale::En, "results_ai_loading") => "Analyzing your personality",
        (Locale::Ru, "results_ai_loading") => "Анализируем вашу личность",

        (Locale::En, "results_ai_loading_hint") => "This usually takes about a minute...",
        (Locale::Ru, "results_ai_loading_hint") => "Обычно это занимает около минуты...",

        (Locale::En, "results_ai_error") => "Failed to generate analysis",
        (Locale::Ru, "results_ai_error") => "Не удалось сгенерировать анализ",

        (Locale::En, "results_ai_retry") => "Try Again",
        (Locale::Ru, "results_ai_retry") => "Попробовать снова",

        (Locale::En, "results_ai_regenerate") => "Regenerate",
        (Locale::Ru, "results_ai_regenerate") => "Сгенерировать заново",

        (Locale::En, "results_context_label") => "Tell us about yourself",
        (Locale::Ru, "results_context_label") => "Расскажите о себе",

        (Locale::En, "results_context_optional") => "(optional)",
        (Locale::Ru, "results_context_optional") => "(необязательно)",

        (Locale::En, "results_context_placeholder") => {
            "A few sentences about your life, work, hobbies, or anything that helps paint a fuller picture..."
        }
        (Locale::Ru, "results_context_placeholder") => {
            "Несколько предложений о вашей жизни, работе, увлечениях — всё, что поможет создать более полную картину..."
        }

        (Locale::En, "results_context_hint") => {
            "Adding context helps the AI provide more personalized and relevant insights."
        }
        (Locale::Ru, "results_context_hint") => {
            "Добавление контекста поможет AI дать более персонализированный и точный анализ."
        }

        (Locale::En, "results_model_select") => "Analysis Model",
        (Locale::Ru, "results_model_select") => "Модель анализа",

        (Locale::En, "results_retake") => "Retake Test",
        (Locale::Ru, "results_retake") => "Пройти тест заново",

        (Locale::En, "results_home") => "Back to Home",
        (Locale::Ru, "results_home") => "На главную",

        (Locale::En, "results_export_pdf") => "Export as PDF",
        (Locale::Ru, "results_export_pdf") => "Экспорт в PDF",

        (Locale::En, "results_copy_link") => "Copy Link",
        (Locale::Ru, "results_copy_link") => "Скопировать ссылку",

        (Locale::En, "results_link_copied") => "Copied!",
        (Locale::Ru, "results_link_copied") => "Скопировано!",

        (Locale::En, "results_share_saving") => "Saving...",
        (Locale::Ru, "results_share_saving") => "Сохранение...",

        (Locale::En, "results_not_found") => "Results not found",
        (Locale::Ru, "results_not_found") => "Результаты не найдены",

        (Locale::En, "results_ai_not_generated") => "AI analysis has not been generated yet.",
        (Locale::Ru, "results_ai_not_generated") => "AI-анализ ещё не был сгенерирован.",

        // Score levels
        (Locale::En, "level_low") => "Low",
        (Locale::Ru, "level_low") => "Низкий",

        (Locale::En, "level_neutral") => "Average",
        (Locale::Ru, "level_neutral") => "Средний",

        (Locale::En, "level_high") => "High",
        (Locale::Ru, "level_high") => "Высокий",

        // Domain names
        (Locale::En, "domain_neuroticism") => "Neuroticism",
        (Locale::Ru, "domain_neuroticism") => "Нейротизм",

        (Locale::En, "domain_extraversion") => "Extraversion",
        (Locale::Ru, "domain_extraversion") => "Экстраверсия",

        (Locale::En, "domain_openness") => "Openness to Experience",
        (Locale::Ru, "domain_openness") => "Открытость опыту",

        (Locale::En, "domain_agreeableness") => "Agreeableness",
        (Locale::Ru, "domain_agreeableness") => "Доброжелательность",

        (Locale::En, "domain_conscientiousness") => "Conscientiousness",
        (Locale::Ru, "domain_conscientiousness") => "Добросовестность",

        // Facet names - Neuroticism
        (Locale::En, "facet_anxiety") => "Anxiety",
        (Locale::Ru, "facet_anxiety") => "Тревожность",

        (Locale::En, "facet_anger") => "Anger",
        (Locale::Ru, "facet_anger") => "Гнев",

        (Locale::En, "facet_depression") => "Depression",
        (Locale::Ru, "facet_depression") => "Депрессия",

        (Locale::En, "facet_self_consciousness") => "Self-Consciousness",
        (Locale::Ru, "facet_self_consciousness") => "Застенчивость",

        (Locale::En, "facet_immoderation") => "Immoderation",
        (Locale::Ru, "facet_immoderation") => "Невоздержанность",

        (Locale::En, "facet_vulnerability") => "Vulnerability",
        (Locale::Ru, "facet_vulnerability") => "Уязвимость",

        // Facet names - Extraversion
        (Locale::En, "facet_friendliness") => "Friendliness",
        (Locale::Ru, "facet_friendliness") => "Дружелюбие",

        (Locale::En, "facet_gregariousness") => "Gregariousness",
        (Locale::Ru, "facet_gregariousness") => "Общительность",

        (Locale::En, "facet_assertiveness") => "Assertiveness",
        (Locale::Ru, "facet_assertiveness") => "Напористость",

        (Locale::En, "facet_activity_level") => "Activity Level",
        (Locale::Ru, "facet_activity_level") => "Активность",

        (Locale::En, "facet_excitement_seeking") => "Excitement-Seeking",
        (Locale::Ru, "facet_excitement_seeking") => "Поиск острых ощущений",

        (Locale::En, "facet_cheerfulness") => "Cheerfulness",
        (Locale::Ru, "facet_cheerfulness") => "Жизнерадостность",

        // Facet names - Openness
        (Locale::En, "facet_imagination") => "Imagination",
        (Locale::Ru, "facet_imagination") => "Воображение",

        (Locale::En, "facet_artistic_interests") => "Artistic Interests",
        (Locale::Ru, "facet_artistic_interests") => "Художественные интересы",

        (Locale::En, "facet_emotionality") => "Emotionality",
        (Locale::Ru, "facet_emotionality") => "Эмоциональность",

        (Locale::En, "facet_adventurousness") => "Adventurousness",
        (Locale::Ru, "facet_adventurousness") => "Авантюризм",

        (Locale::En, "facet_intellect") => "Intellect",
        (Locale::Ru, "facet_intellect") => "Интеллект",

        (Locale::En, "facet_liberalism") => "Liberalism",
        (Locale::Ru, "facet_liberalism") => "Либерализм",

        // Facet names - Agreeableness
        (Locale::En, "facet_trust") => "Trust",
        (Locale::Ru, "facet_trust") => "Доверие",

        (Locale::En, "facet_morality") => "Morality",
        (Locale::Ru, "facet_morality") => "Нравственность",

        (Locale::En, "facet_altruism") => "Altruism",
        (Locale::Ru, "facet_altruism") => "Альтруизм",

        (Locale::En, "facet_cooperation") => "Cooperation",
        (Locale::Ru, "facet_cooperation") => "Сотрудничество",

        (Locale::En, "facet_modesty") => "Modesty",
        (Locale::Ru, "facet_modesty") => "Скромность",

        (Locale::En, "facet_sympathy") => "Sympathy",
        (Locale::Ru, "facet_sympathy") => "Сочувствие",

        // Facet names - Conscientiousness
        (Locale::En, "facet_self_efficacy") => "Self-Efficacy",
        (Locale::Ru, "facet_self_efficacy") => "Самоэффективность",

        (Locale::En, "facet_orderliness") => "Orderliness",
        (Locale::Ru, "facet_orderliness") => "Упорядоченность",

        (Locale::En, "facet_dutifulness") => "Dutifulness",
        (Locale::Ru, "facet_dutifulness") => "Чувство долга",

        (Locale::En, "facet_achievement_striving") => "Achievement-Striving",
        (Locale::Ru, "facet_achievement_striving") => "Стремление к достижениям",

        (Locale::En, "facet_self_discipline") => "Self-Discipline",
        (Locale::Ru, "facet_self_discipline") => "Самодисциплина",

        (Locale::En, "facet_cautiousness") => "Cautiousness",
        (Locale::Ru, "facet_cautiousness") => "Осторожность",

        // Fallback - return empty string for unknown keys
        (_, _key) => {
            #[cfg(debug_assertions)]
            leptos::logging::warn!("Missing translation for key: {}", _key);
            "[?]"
        }
    }
}

/// Helper trait to easily get translations from i18n context.
impl I18nContext {
    /// Get a translation string.
    pub fn t(&self, key: &str) -> &'static str {
        t(self.get_locale(), key)
    }
}
