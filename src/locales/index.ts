import { createI18n } from 'vue-i18n'
import en from './en'
import zh from './zh'

// 支持的语言列表
export const supportedLocales = [
    { code: 'en', name: 'English' },
    { code: 'zh', name: '中文' },
]

const savedLocale = localStorage.getItem('eve-wrench-locale')
const initialLocale = supportedLocales.some(
    (locale) => locale.code === savedLocale
)
    ? savedLocale!
    : 'en'

// 创建i18n实例
const i18n = createI18n({
    legacy: false, // 使用组合式API
    locale: initialLocale,
    fallbackLocale: 'en', // 回退语言
    messages: {
        en,
        zh,
    },
})

export default i18n
