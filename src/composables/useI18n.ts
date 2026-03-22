import { useI18n as useVueI18n } from 'vue-i18n'
import { supportedLocales } from '../locales'
import { ref, watch } from 'vue'

/**
 * 国际化管理组合式函数
 * 提供语言切换和当前语言信息
 */
export function useI18n() {
  const { locale, t } = useVueI18n()
  
  // 当前选择的语言
  const currentLocale = ref(locale.value)
  
  // 支持的语言列表
  const languages = supportedLocales
  
  // 切换语言
  const changeLanguage = (lang: string) => {
    currentLocale.value = lang
    locale.value = lang
  }
  
  // 监听语言变化
  watch(locale, (newLocale) => {
    currentLocale.value = newLocale
  })
  
  return {
    locale: currentLocale,
    languages,
    t,
    changeLanguage
  }
}
