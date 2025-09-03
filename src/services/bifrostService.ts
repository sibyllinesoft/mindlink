// Bifrost API Service for logs and usage statistics

export interface BifrostLogEntry {
  id: string
  timestamp: string
  method: string
  path: string
  status: number
  provider?: string
  model?: string
  tokens_used?: number
  latency_ms?: number
  virtual_key?: string
  request_content?: string
  response_content?: string
  error?: string
}

export interface BifrostLogsResponse {
  logs: BifrostLogEntry[]
  total_count: number
  page: number
  limit: number
}

export interface LogsQueryParams {
  content_search?: string
  start_time?: string
  end_time?: string
  limit?: number
  page?: number
  provider?: string
  model?: string
  status?: number
  min_tokens?: number
  max_tokens?: number
}

export interface AppUsageStats {
  totalRequests: number
  requestsToday: number
  requestsThisWeek: number
  requestsThisMonth: number
  totalTokens: number
  averageLatency: number
  successRate: number
  lastUsed: string
  providers: { [key: string]: number }
  models: { [key: string]: number }
}

class BifrostService {
  private baseUrl: string

  constructor(baseUrl: string = 'http://127.0.0.1:3003') {
    this.baseUrl = baseUrl
  }

  async getLogs(params: LogsQueryParams): Promise<BifrostLogsResponse> {
    const queryString = new URLSearchParams()
    
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined) {
        queryString.append(key, String(value))
      }
    })

    const response = await fetch(`${this.baseUrl}/api/logs?${queryString}`)
    
    if (!response.ok) {
      throw new Error(`Failed to fetch logs: ${response.statusText}`)
    }

    return await response.json()
  }

  async getAppLogs(virtualKey: string, limit: number = 100): Promise<BifrostLogEntry[]> {
    const thirtyDaysAgo = new Date()
    thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30)

    const params: LogsQueryParams = {
      content_search: virtualKey,
      start_time: thirtyDaysAgo.toISOString(),
      end_time: new Date().toISOString(),
      limit
    }

    const response = await this.getLogs(params)
    return response.logs
  }

  async getAppUsageStats(virtualKey: string): Promise<AppUsageStats> {
    // Get logs for the past 30 days
    const logs = await this.getAppLogs(virtualKey, 1000)
    
    if (logs.length === 0) {
      return {
        totalRequests: 0,
        requestsToday: 0,
        requestsThisWeek: 0,
        requestsThisMonth: 0,
        totalTokens: 0,
        averageLatency: 0,
        successRate: 0,
        lastUsed: '',
        providers: {},
        models: {}
      }
    }

    const now = new Date()
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
    const thisWeek = new Date(now.getTime() - (7 * 24 * 60 * 60 * 1000))
    const thisMonth = new Date(now.getTime() - (30 * 24 * 60 * 60 * 1000))

    const stats: AppUsageStats = {
      totalRequests: logs.length,
      requestsToday: 0,
      requestsThisWeek: 0,
      requestsThisMonth: 0,
      totalTokens: 0,
      averageLatency: 0,
      successRate: 0,
      lastUsed: '',
      providers: {},
      models: {}
    }

    let totalLatency = 0
    let successCount = 0
    let latencyCount = 0

    logs.forEach(log => {
      const logDate = new Date(log.timestamp)
      
      // Count requests by time period
      if (logDate >= today) {
        stats.requestsToday++
      }
      if (logDate >= thisWeek) {
        stats.requestsThisWeek++
      }
      if (logDate >= thisMonth) {
        stats.requestsThisMonth++
      }

      // Accumulate tokens
      if (log.tokens_used) {
        stats.totalTokens += log.tokens_used
      }

      // Accumulate latency
      if (log.latency_ms) {
        totalLatency += log.latency_ms
        latencyCount++
      }

      // Count successes (2xx status codes)
      if (log.status >= 200 && log.status < 300) {
        successCount++
      }

      // Track providers
      if (log.provider) {
        stats.providers[log.provider] = (stats.providers[log.provider] || 0) + 1
      }

      // Track models
      if (log.model) {
        stats.models[log.model] = (stats.models[log.model] || 0) + 1
      }

      // Update last used
      if (!stats.lastUsed || logDate > new Date(stats.lastUsed)) {
        stats.lastUsed = log.timestamp
      }
    })

    // Calculate averages and percentages
    stats.averageLatency = latencyCount > 0 ? Math.round(totalLatency / latencyCount) : 0
    stats.successRate = logs.length > 0 ? Math.round((successCount / logs.length) * 100) : 0

    return stats
  }

  async getRecentActivity(virtualKey: string, hours: number = 24): Promise<BifrostLogEntry[]> {
    const startTime = new Date()
    startTime.setHours(startTime.getHours() - hours)

    const params: LogsQueryParams = {
      content_search: virtualKey,
      start_time: startTime.toISOString(),
      end_time: new Date().toISOString(),
      limit: 50
    }

    const response = await this.getLogs(params)
    return response.logs
  }

  // Format timestamps for display
  formatTimestamp(timestamp: string): string {
    return new Date(timestamp).toLocaleString()
  }

  // Format latency for display
  formatLatency(latencyMs: number): string {
    if (latencyMs < 1000) {
      return `${latencyMs}ms`
    }
    return `${(latencyMs / 1000).toFixed(1)}s`
  }

  // Format tokens for display
  formatTokens(tokens: number): string {
    if (tokens < 1000) {
      return tokens.toString()
    }
    if (tokens < 1000000) {
      return `${(tokens / 1000).toFixed(1)}K`
    }
    return `${(tokens / 1000000).toFixed(1)}M`
  }
}

export const bifrostService = new BifrostService()
export default BifrostService