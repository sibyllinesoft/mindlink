const fastify = require('fastify');
const axios = require('axios');
const { v4: uuidv4 } = require('uuid');

/**
 * ServerManager handles the local OpenAI-compatible API server
 * Using Fastify for better performance and modern async/await support
 */
class ServerManager {
    constructor() {
        this.app = null;
        this.port = 3001;
        this.host = '127.0.0.1';
        this.isRunning = false;
        this.authManager = null;
        this.baseInstructions = this.getBaseInstructions();
        this.chatgptResponsesUrl = 'https://chatgpt.com/backend-api/codex/responses';
    }

    setAuthManager(authManager) {
        this.authManager = authManager;
    }

    async start() {
        if (this.isRunning) return;

        this.createApp();
        await this.setupRoutes();
        
        try {
            await this.app.listen({ 
                port: this.port, 
                host: this.host 
            });
            
            this.isRunning = true;
            console.log(`MindLink server running on ${this.getLocalUrl()}`);
            
        } catch (err) {
            console.error('Failed to start server:', err);
            throw err;
        }
    }

    async stop() {
        if (!this.isRunning || !this.app) return;

        try {
            await this.app.close();
            this.isRunning = false;
            this.app = null;
            console.log('MindLink server stopped');
        } catch (err) {
            console.error('Error stopping server:', err);
        }
    }

    createApp() {
        this.app = fastify({
            logger: {
                level: 'info'
            },
            bodyLimit: 52428800 // 50MB
        });

        // Register CORS plugin
        this.app.register(require('@fastify/cors'), {
            origin: true,
            credentials: true,
            methods: ['GET', 'POST', 'OPTIONS']
        });

        // Global error handler
        this.app.setErrorHandler(async (error, request, reply) => {
            console.error('Server error:', error);
            return reply.status(500).send({
                error: {
                    message: 'Internal server error',
                    type: 'server_error'
                }
            });
        });
    }

    async setupRoutes() {
        // Health check routes
        this.app.get('/', async (request, reply) => {
            return {
                status: 'ok',
                service: 'MindLink API',
                version: '1.0.0',
                timestamp: new Date().toISOString()
            };
        });

        this.app.get('/health', async (request, reply) => {
            return {
                status: 'healthy',
                authenticated: this.authManager?.isAuthenticated() || false,
                uptime: process.uptime()
            };
        });

        // OpenAI Compatible API Routes
        this.app.get('/v1/models', async (request, reply) => {
            return this.handleListModels();
        });

        this.app.post('/v1/chat/completions', async (request, reply) => {
            return this.handleChatCompletions(request, reply);
        });

        this.app.post('/v1/completions', async (request, reply) => {
            return this.handleCompletions(request, reply);
        });

        // Web interface
        this.app.get('/dashboard', async (request, reply) => {
            reply.type('text/html');
            return this.getDashboardHtml();
        });
    }

    async handleListModels() {
        const models = {
            object: 'list',
            data: [
                {
                    id: 'gpt-5',
                    object: 'model',
                    created: Date.now(),
                    owned_by: 'openai'
                },
                {
                    id: 'codex-mini',
                    object: 'model',
                    created: Date.now(),
                    owned_by: 'openai'
                }
            ]
        };

        res.json(models);
    }

    async handleChatCompletions(request, reply) {
        try {
            // Validate authentication
            const authTokens = this.getAuthTokens();
            if (!authTokens) {
                return reply.status(401).send({
                    error: {
                        message: 'Authentication required. Please login first.',
                        type: 'authentication_error',
                        code: 'invalid_api_key'
                    }
                });
            }

            const { model, messages, tools, tool_choice, stream = false, ...otherParams } = request.body;

            // Validate request
            if (!messages || !Array.isArray(messages)) {
                return reply.status(400).send({
                    error: {
                        message: 'Missing required parameter: messages',
                        type: 'invalid_request_error'
                    }
                });
            }

            // Normalize model name
            const normalizedModel = this.normalizeModelName(model);
            
            // Convert messages to ChatGPT format
            const inputItems = this.convertMessagesToResponsesInput(messages);
            const toolsFormatted = this.convertToolsToResponses(tools);

            // Build request payload for ChatGPT
            const payload = {
                model: normalizedModel,
                instructions: this.baseInstructions,
                input: inputItems,
                tools: toolsFormatted || [],
                tool_choice: tool_choice || 'auto',
                parallel_tool_calls: otherParams.parallel_tool_calls || false,
                store: false,
                stream: true,
                include: []
            };

            // Add reasoning parameters if available
            const reasoning = this.buildReasoningParam(otherParams.reasoning);
            if (reasoning) {
                payload.reasoning = reasoning;
                if (reasoning.effort !== 'none') {
                    payload.include.push('reasoning.encrypted_content');
                }
            }

            // Make request to ChatGPT
            const upstreamResponse = await this.makeUpstreamRequest(payload, authTokens);

            if (stream) {
                // Handle streaming response
                reply.raw.writeHead(200, {
                    'Content-Type': 'text/event-stream',
                    'Cache-Control': 'no-cache',
                    'Connection': 'keep-alive',
                    'Access-Control-Allow-Origin': '*'
                });
                this.handleStreamingResponse(upstreamResponse, normalizedModel, reply.raw);
            } else {
                // Handle non-streaming response
                const completion = await this.handleNonStreamingResponse(upstreamResponse, normalizedModel);
                return completion;
            }

        } catch (error) {
            console.error('Chat completions error:', error);
            return reply.status(500).send({
                error: {
                    message: error.message || 'Internal server error',
                    type: 'server_error'
                }
            });
        }
    }

    async handleCompletions(request, reply) {
        try {
            // Convert completion request to chat format
            const { prompt, model, stream = false, ...otherParams } = request.body;
            
            const messages = [
                { role: 'user', content: prompt || '' }
            ];

            // Delegate to chat completions handler
            const modifiedRequest = {
                ...request,
                body: {
                    model,
                    messages,
                    stream,
                    ...otherParams
                }
            };

            return this.handleChatCompletions(modifiedRequest, reply);

        } catch (error) {
            console.error('Completions error:', error);
            return reply.status(500).send({
                error: {
                    message: error.message || 'Internal server error',
                    type: 'server_error'
                }
            });
        }
    }

    async makeUpstreamRequest(payload, authTokens) {
        const headers = {
            'Authorization': `Bearer ${authTokens.accessToken}`,
            'Content-Type': 'application/json',
            'Accept': 'text/event-stream',
            'chatgpt-account-id': authTokens.accountId,
            'OpenAI-Beta': 'responses=experimental'
        };

        try {
            const response = await axios.post(this.chatgptResponsesUrl, payload, {
                headers,
                responseType: 'stream',
                timeout: 600000 // 10 minutes
            });

            return response;

        } catch (error) {
            if (error.response?.status === 401) {
                // Try to refresh tokens
                try {
                    await this.authManager.refreshTokens();
                    const newTokens = this.authManager.getAuthTokens();
                    headers['Authorization'] = `Bearer ${newTokens.accessToken}`;
                    headers['chatgpt-account-id'] = newTokens.accountId;
                    
                    return axios.post(this.chatgptResponsesUrl, payload, {
                        headers,
                        responseType: 'stream',
                        timeout: 600000
                    });
                } catch (refreshError) {
                    throw new Error('Authentication expired. Please login again.');
                }
            }
            throw error;
        }
    }

    handleStreamingResponse(upstreamResponse, model, res) {
        const created = Math.floor(Date.now() / 1000);
        const responseId = `chatcmpl-${uuidv4().slice(0, 8)}`;

        res.writeHead(200, {
            'Content-Type': 'text/event-stream',
            'Cache-Control': 'no-cache',
            'Connection': 'keep-alive',
            'Access-Control-Allow-Origin': '*'
        });

        let fullText = '';
        let thinkOpen = false;
        let thinkClosed = false;

        upstreamResponse.data.on('data', (chunk) => {
            const lines = chunk.toString().split('\n');
            
            for (const line of lines) {
                if (!line.startsWith('data: ')) continue;
                
                const data = line.slice(6).trim();
                if (!data || data === '[DONE]') {
                    if (data === '[DONE]') {
                        res.write('data: [DONE]\n\n');
                    }
                    continue;
                }

                try {
                    const event = JSON.parse(data);
                    const kind = event.type;

                    if (kind === 'response.output_text.delta') {
                        const delta = event.delta || '';
                        
                        if (thinkOpen && !thinkClosed) {
                            const closeChunk = {
                                id: responseId,
                                object: 'chat.completion.chunk',
                                created,
                                model,
                                choices: [{
                                    index: 0,
                                    delta: { content: '</think>' },
                                    finish_reason: null
                                }]
                            };
                            res.write(`data: ${JSON.stringify(closeChunk)}\n\n`);
                            thinkOpen = false;
                            thinkClosed = true;
                        }

                        const chunk = {
                            id: responseId,
                            object: 'chat.completion.chunk',
                            created,
                            model,
                            choices: [{
                                index: 0,
                                delta: { content: delta },
                                finish_reason: null
                            }]
                        };
                        res.write(`data: ${JSON.stringify(chunk)}\n\n`);
                        fullText += delta;

                    } else if (kind === 'response.reasoning_summary_text.delta' || kind === 'response.reasoning_text.delta') {
                        const delta = event.delta || '';
                        
                        if (!thinkOpen && !thinkClosed) {
                            const openChunk = {
                                id: responseId,
                                object: 'chat.completion.chunk',
                                created,
                                model,
                                choices: [{
                                    index: 0,
                                    delta: { content: '<think>' },
                                    finish_reason: null
                                }]
                            };
                            res.write(`data: ${JSON.stringify(openChunk)}\n\n`);
                            thinkOpen = true;
                        }

                        if (thinkOpen && !thinkClosed) {
                            const chunk = {
                                id: responseId,
                                object: 'chat.completion.chunk',
                                created,
                                model,
                                choices: [{
                                    index: 0,
                                    delta: { content: delta },
                                    finish_reason: null
                                }]
                            };
                            res.write(`data: ${JSON.stringify(chunk)}\n\n`);
                        }

                    } else if (kind === 'response.completed') {
                        if (thinkOpen && !thinkClosed) {
                            const closeChunk = {
                                id: responseId,
                                object: 'chat.completion.chunk',
                                created,
                                model,
                                choices: [{
                                    index: 0,
                                    delta: { content: '</think>' },
                                    finish_reason: null
                                }]
                            };
                            res.write(`data: ${JSON.stringify(closeChunk)}\n\n`);
                        }

                        const finalChunk = {
                            id: responseId,
                            object: 'chat.completion.chunk',
                            created,
                            model,
                            choices: [{
                                index: 0,
                                delta: {},
                                finish_reason: 'stop'
                            }]
                        };
                        res.write(`data: ${JSON.stringify(finalChunk)}\n\n`);
                        res.write('data: [DONE]\n\n');
                        res.end();
                        return;
                    }

                } catch (parseError) {
                    console.error('Error parsing SSE data:', parseError);
                }
            }
        });

        upstreamResponse.data.on('error', (error) => {
            console.error('Upstream stream error:', error);
            res.write(`data: ${JSON.stringify({ error: { message: error.message } })}\n\n`);
            res.end();
        });

        upstreamResponse.data.on('end', () => {
            res.end();
        });
    }

    async handleNonStreamingResponse(upstreamResponse, model) {
        const created = Math.floor(Date.now() / 1000);
        const responseId = `chatcmpl-${uuidv4().slice(0, 8)}`;

        let fullText = '';
        let reasoningText = '';
        let toolCalls = [];

        return new Promise((resolve, reject) => {
            upstreamResponse.data.on('data', (chunk) => {
                const lines = chunk.toString().split('\n');
                
                for (const line of lines) {
                    if (!line.startsWith('data: ')) continue;
                    
                    const data = line.slice(6).trim();
                    if (!data || data === '[DONE]') continue;

                    try {
                        const event = JSON.parse(data);
                        const kind = event.type;

                        if (kind === 'response.output_text.delta') {
                            fullText += event.delta || '';
                        } else if (kind === 'response.reasoning_summary_text.delta' || kind === 'response.reasoning_text.delta') {
                            reasoningText += event.delta || '';
                        } else if (kind === 'response.output_item.done') {
                            const item = event.item || {};
                            if (item.type === 'function_call') {
                                toolCalls.push({
                                    id: item.call_id || item.id,
                                    type: 'function',
                                    function: {
                                        name: item.name,
                                        arguments: item.arguments
                                    }
                                });
                            }
                        } else if (kind === 'response.completed') {
                            const message = {
                                role: 'assistant',
                                content: reasoningText ? `<think>${reasoningText}</think>${fullText}` : fullText
                            };

                            if (toolCalls.length > 0) {
                                message.tool_calls = toolCalls;
                            }

                            const completion = {
                                id: responseId,
                                object: 'chat.completion',
                                created,
                                model,
                                choices: [{
                                    index: 0,
                                    message,
                                    finish_reason: 'stop'
                                }]
                            };

                            resolve(completion);
                            return;
                        }

                    } catch (parseError) {
                        console.error('Error parsing response data:', parseError);
                    }
                }
            });

            upstreamResponse.data.on('error', reject);
        });
    }

    getAuthTokens() {
        return this.authManager?.getAuthTokens() || null;
    }

    normalizeModelName(model) {
        if (!model) return 'gpt-5';
        
        const base = model.split(':')[0];
        const mapping = {
            'gpt5': 'gpt-5',
            'gpt-5-latest': 'gpt-5',
            'gpt-5': 'gpt-5',
            'codex': 'codex-mini-latest',
            'codex-mini': 'codex-mini-latest',
            'codex-mini-latest': 'codex-mini-latest'
        };
        
        return mapping[base] || base;
    }

    convertMessagesToResponsesInput(messages) {
        const inputItems = [];
        
        for (const message of messages) {
            const role = message.role;
            if (role === 'system') continue; // Skip system messages
            
            // Handle tool messages
            if (role === 'tool') {
                const callId = message.tool_call_id || message.id;
                if (callId) {
                    inputItems.push({
                        type: 'function_call_output',
                        call_id: callId,
                        output: message.content || ''
                    });
                }
                continue;
            }

            // Handle assistant tool calls
            if (role === 'assistant' && message.tool_calls) {
                for (const toolCall of message.tool_calls) {
                    if (toolCall.type === 'function') {
                        inputItems.push({
                            type: 'function_call',
                            name: toolCall.function.name,
                            arguments: toolCall.function.arguments,
                            call_id: toolCall.id
                        });
                    }
                }
            }

            // Handle regular message content
            const content = message.content;
            const contentItems = [];
            
            if (Array.isArray(content)) {
                for (const part of content) {
                    if (part.type === 'text') {
                        const textType = role === 'assistant' ? 'output_text' : 'input_text';
                        contentItems.push({
                            type: textType,
                            text: part.text
                        });
                    } else if (part.type === 'image_url') {
                        contentItems.push({
                            type: 'input_image',
                            image_url: part.image_url.url
                        });
                    }
                }
            } else if (typeof content === 'string' && content) {
                const textType = role === 'assistant' ? 'output_text' : 'input_text';
                contentItems.push({
                    type: textType,
                    text: content
                });
            }

            if (contentItems.length > 0) {
                const roleOut = role === 'assistant' ? 'assistant' : 'user';
                inputItems.push({
                    type: 'message',
                    role: roleOut,
                    content: contentItems
                });
            }
        }
        
        return inputItems;
    }

    convertToolsToResponses(tools) {
        if (!Array.isArray(tools)) return null;
        
        return tools
            .filter(tool => tool.type === 'function')
            .map(tool => ({
                type: 'function',
                name: tool.function.name,
                description: tool.function.description || '',
                strict: false,
                parameters: tool.function.parameters || { type: 'object', properties: {} }
            }));
    }

    buildReasoningParam(reasoning) {
        const defaultReasoning = {
            effort: 'medium',
            summary: 'auto'
        };

        if (reasoning && typeof reasoning === 'object') {
            return {
                effort: reasoning.effort || defaultReasoning.effort,
                summary: reasoning.summary || defaultReasoning.summary
            };
        }

        return defaultReasoning;
    }

    async checkHealth() {
        if (!this.isRunning) return false;

        try {
            const response = await axios.get(`${this.getLocalUrl()}/health`, {
                timeout: 5000
            });
            return response.status === 200;
        } catch (error) {
            console.error('Health check failed:', error.message);
            return false;
        }
    }

    getLocalUrl() {
        return `http://${this.host}:${this.port}`;
    }

    getBaseInstructions() {
        return `You are a helpful AI assistant. Respond to user queries accurately and concisely.`;
    }

    getDashboardHtml() {
        return `
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>MindLink Dashboard</title>
                <style>
                    body { 
                        font-family: system-ui, -apple-system, sans-serif; 
                        max-width: 800px; 
                        margin: 0 auto; 
                        padding: 20px; 
                        background: #f8f9fa; 
                    }
                    .card { 
                        background: white; 
                        border-radius: 8px; 
                        padding: 20px; 
                        margin: 20px 0; 
                        box-shadow: 0 2px 4px rgba(0,0,0,0.1); 
                    }
                    .status { 
                        display: inline-block; 
                        padding: 4px 12px; 
                        border-radius: 20px; 
                        font-size: 14px; 
                        font-weight: 500; 
                    }
                    .status.connected { background: #d4edda; color: #155724; }
                    .status.disconnected { background: #f8d7da; color: #721c24; }
                    code { 
                        background: #f8f9fa; 
                        padding: 2px 6px; 
                        border-radius: 4px; 
                        font-family: 'SF Mono', 'Monaco', monospace; 
                    }
                    .endpoint { 
                        background: #e3f2fd; 
                        padding: 12px; 
                        border-radius: 4px; 
                        margin: 10px 0; 
                    }
                    .copy-btn {
                        background: #007bff;
                        color: white;
                        border: none;
                        padding: 6px 12px;
                        border-radius: 4px;
                        cursor: pointer;
                        font-size: 12px;
                        margin-left: 10px;
                    }
                    .copy-btn:hover { background: #0056b3; }
                </style>
            </head>
            <body>
                <h1>🔗 MindLink Dashboard</h1>
                
                <div class="card">
                    <h3>Service Status</h3>
                    <span class="status connected">🟢 Active</span>
                    <p>Your local LLM API bridge is running and ready to accept requests.</p>
                </div>

                <div class="card">
                    <h3>API Endpoints</h3>
                    
                    <div class="endpoint">
                        <strong>Base URL:</strong> 
                        <code id="baseUrl">${this.getLocalUrl()}</code>
                        <button class="copy-btn" onclick="copyToClipboard('baseUrl')">Copy</button>
                    </div>

                    <div class="endpoint">
                        <strong>OpenAI Compatible:</strong> 
                        <code id="openaiUrl">${this.getLocalUrl()}/v1</code>
                        <button class="copy-btn" onclick="copyToClipboard('openaiUrl')">Copy</button>
                    </div>
                    
                    <h4>Available Models:</h4>
                    <ul>
                        <li><code>gpt-5</code> - Latest GPT-5 model</li>
                        <li><code>codex-mini</code> - Code-focused model</li>
                    </ul>
                </div>

                <div class="card">
                    <h3>Usage Example</h3>
                    <pre><code>curl ${this.getLocalUrl()}/v1/chat/completions \\
  -H "Authorization: Bearer any-key" \\
  -H "Content-Type: application/json" \\
  -d '{
    "model": "gpt-5",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'</code></pre>
                </div>

                <script>
                    function copyToClipboard(elementId) {
                        const element = document.getElementById(elementId);
                        navigator.clipboard.writeText(element.textContent).then(() => {
                            const btn = element.nextElementSibling;
                            const originalText = btn.textContent;
                            btn.textContent = 'Copied!';
                            setTimeout(() => {
                                btn.textContent = originalText;
                            }, 2000);
                        });
                    }
                </script>
            </body>
            </html>
        `;
    }
}

module.exports = ServerManager;