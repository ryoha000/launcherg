// 基本抽出システム - 設定ベースでHTML解析を行う

export interface ExtractionRule {
  name?: string
  selectors?: string[]
  selector?: string
  attribute?: string
  fallbackAttribute?: string
  processor?: 'text' | 'html' | 'attr'
  urlPattern?: string
  required: boolean
  description?: string
}

export interface SiteConfig {
  name: string
  domain: string
  detectionRules: ExtractionRule[]
  gameExtractionRules: {
    container: string
    fields: Record<string, ExtractionRule>
  }
}

export interface ExtractedGameData {
  store_id: string
  title: string
  purchase_url: string
  purchase_date?: string
  thumbnail_url?: string
  additional_data: Record<string, string>
}

export class BaseExtractor {
  private config: SiteConfig
  private debugMode: boolean

  constructor(config: SiteConfig, debugMode = false) {
    this.config = config
    this.debugMode = debugMode
  }

  /**
   * 現在のページが対象ページかどうかを判定
   */
  detectPage(): boolean {
    for (const rule of this.config.detectionRules) {
      const element = this.findElement(rule)
      if (rule.required && !element) {
        this.debug(`Required detection rule failed: ${rule.name}`)
        return false
      }
      if (element) {
        this.debug(`Detection rule matched: ${rule.name}`)
        return true
      }
    }
    return false
  }

  /**
   * ページからゲーム情報を抽出
   */
  extractGames(): ExtractedGameData[] {
    const containers = document.querySelectorAll(this.config.gameExtractionRules.container)
    this.debug(`Found ${containers.length} game containers`)

    const games: ExtractedGameData[] = []

    containers.forEach((container, index) => {
      try {
        const gameData = this.extractSingleGame(container as HTMLElement)
        if (gameData) {
          games.push(gameData)
          this.debug(`Extracted game ${index + 1}:`, gameData)
        }
      }
      catch (error) {
        this.debug(`Error extracting game ${index + 1}:`, error)
      }
    })

    return games
  }

  /**
   * 単一のゲーム要素から情報を抽出
   */
  private extractSingleGame(container: HTMLElement): ExtractedGameData | null {
    const fields = this.config.gameExtractionRules.fields
    const gameData: Partial<ExtractedGameData> = {
      additional_data: {},
    }

    // 各フィールドを抽出
    for (const [fieldName, rule] of Object.entries(fields)) {
      try {
        const value = this.extractField(container, rule)
        if (value) {
          if (fieldName === 'store_id' || fieldName === 'title' || fieldName === 'purchase_url') {
            gameData[fieldName] = value
          }
          else if (fieldName === 'purchase_date' || fieldName === 'thumbnail_url') {
            gameData[fieldName] = value
          }
          else {
            gameData.additional_data![fieldName] = value
          }
        }
        else if (rule.required) {
          this.debug(`Required field missing: ${fieldName}`)
          return null
        }
      }
      catch (error) {
        this.debug(`Error extracting field ${fieldName}:`, error)
        if (rule.required) {
          return null
        }
      }
    }

    // 必須フィールドの確認
    if (!gameData.store_id || !gameData.title || !gameData.purchase_url) {
      this.debug('Missing required fields:', gameData)
      return null
    }

    return gameData as ExtractedGameData
  }

  /**
   * 指定されたルールに基づいてフィールドを抽出
   */
  private extractField(container: HTMLElement, rule: ExtractionRule): string | null {
    const element = this.findElement(rule, container)
    if (!element) {
      return null
    }

    let value: string | null = null

    // 値の抽出方法を決定
    if (rule.attribute) {
      value = element.getAttribute(rule.attribute)
      if (!value && rule.fallbackAttribute) {
        value = element.getAttribute(rule.fallbackAttribute)
      }
    }
    else {
      switch (rule.processor) {
        case 'html':
          value = element.innerHTML
          break
        case 'text':
        default:
          value = element.textContent
          break
      }
    }

    // URLパターンからの抽出
    if (!value && rule.urlPattern && rule.attribute === 'href') {
      const href = element.getAttribute('href')
      if (href) {
        const match = href.match(new RegExp(rule.urlPattern))
        if (match && match[1]) {
          value = match[1]
        }
      }
    }

    // フォールバック属性の使用
    if (!value && rule.fallbackAttribute) {
      value = element.getAttribute(rule.fallbackAttribute)
    }

    return value ? value.trim() : null
  }

  /**
   * ルールに基づいて要素を検索
   */
  private findElement(rule: ExtractionRule, container: HTMLElement | Document = document): Element | null {
    const selectors = rule.selectors || (rule.selector ? [rule.selector] : [])

    for (const selector of selectors) {
      try {
        const element = container.querySelector(selector)
        if (element) {
          return element
        }
      }
      catch (error) {
        this.debug(`Invalid selector: ${selector}`, error)
      }
    }

    return null
  }

  /**
   * デバッグログ出力
   */
  private debug(message: string, ...args: any[]): void {
    if (this.debugMode) {
      console.log(`[BaseExtractor] ${message}`, ...args)
    }
  }

  /**
   * 設定の検証
   */
  validateConfig(): boolean {
    try {
      // 基本的な設定の存在確認
      if (!this.config.name || !this.config.domain) {
        throw new Error('Missing basic config properties')
      }

      // 検出ルールの確認
      if (!this.config.detectionRules || this.config.detectionRules.length === 0) {
        throw new Error('No detection rules defined')
      }

      // ゲーム抽出ルールの確認
      const gameRules = this.config.gameExtractionRules
      if (!gameRules || !gameRules.container || !gameRules.fields) {
        throw new Error('Invalid game extraction rules')
      }

      // 必須フィールドの確認
      const requiredFields = ['store_id', 'title', 'purchase_url']
      for (const field of requiredFields) {
        if (!gameRules.fields[field]) {
          throw new Error(`Missing required field: ${field}`)
        }
      }

      return true
    }
    catch (error) {
      this.debug('Config validation failed:', error)
      return false
    }
  }
}

/**
 * HTMLサンプルからルールを自動生成するヘルパー
 */
export class RuleGenerator {
  static analyzeHTML(html: string): Partial<SiteConfig> {
    const parser = new DOMParser()
    const doc = parser.parseFromString(html, 'text/html')

    // 一般的なパターンを検索
    const suggestions: Partial<SiteConfig> = {
      detectionRules: [],
      gameExtractionRules: {
        container: '',
        fields: {},
      },
    }

    // ゲームコンテナの候補を検索
    const containerCandidates = [
      '.game-item',
      '.product-item',
      '.library-item',
      '[data-game-id]',
      '[data-product-id]',
      '[data-work-id]',
      '.item',
      '.card',
    ]

    for (const candidate of containerCandidates) {
      const elements = doc.querySelectorAll(candidate)
      if (elements.length > 1) {
        suggestions.gameExtractionRules!.container = candidate
        break
      }
    }

    return suggestions
  }

  static generateSelectors(element: HTMLElement): string[] {
    const selectors: string[] = []

    // ID
    if (element.id) {
      selectors.push(`#${element.id}`)
    }

    // クラス
    if (element.className) {
      const classes = element.className.split(' ').filter(c => c.trim())
      if (classes.length > 0) {
        selectors.push(`.${classes.join('.')}`)
        selectors.push(`.${classes[0]}`)
      }
    }

    // データ属性
    for (let i = 0; i < element.attributes.length; i++) {
      const attr = element.attributes[i]
      if (attr.name.startsWith('data-')) {
        selectors.push(`[${attr.name}]`)
        if (attr.value) {
          selectors.push(`[${attr.name}="${attr.value}"]`)
        }
      }
    }

    // タグ名 + クラス
    if (element.className) {
      const classes = element.className.split(' ').filter(c => c.trim())
      if (classes.length > 0) {
        selectors.push(`${element.tagName.toLowerCase()}.${classes[0]}`)
      }
    }

    return selectors
  }
}
