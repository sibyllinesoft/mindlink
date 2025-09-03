# MindLink API Reference

## Overview

MindLink provides a fully OpenAI-compatible API that serves as a bridge to your ChatGPT Plus/Pro account. The API is available both locally and through secure Cloudflare tunnels, enabling seamless integration with existing OpenAI-based applications.

## Base URLs

- **Local API**: `http://localhost:3001/v1` (default, configurable)
- **Public API**: `https://your-unique-id.trycloudflare.com/v1` (generated tunnel URL)
- **Dashboard**: `http://localhost:3001/dashboard` (management interface)

## Authentication

MindLink uses OAuth2 authentication with your ChatGPT account. API requests do not require traditional API keys:

```bash
# Any string can be used as API key - it's ignored
curl -H "Authorization: Bearer any-key" \
     -H "Content-Type: application/json" \
     "$MINDLINK_URL/v1/models"
```

The actual authentication happens through your ChatGPT session managed by MindLink.

## Supported Endpoints

### Models

#### List Available Models
```http
GET /v1/models
```

Returns the list of models available through your ChatGPT subscription.

**Response:**
```json
{
  "object": "list",
  "data": [
    {
      "id": "gpt-5",
      "object": "model",
      "created": 1677610602,
      "owned_by": "openai"
    },
    {
      "id": "gpt-4o",
      "object": "model", 
      "created": 1677610602,
      "owned_by": "openai"
    },
    {
      "id": "gpt-4",
      "object": "model",
      "created": 1677610602,
      "owned_by": "openai"
    },
    {
      "id": "gpt-3.5-turbo",
      "object": "model",
      "created": 1677610602,
      "owned_by": "openai"
    }
  ]
}
```

#### Get Model Details
```http
GET /v1/models/{model_id}
```

**Parameters:**
- `model_id` (string): The model identifier (e.g., "gpt-5", "gpt-4o")

**Response:**
```json
{
  "id": "gpt-5",
  "object": "model",
  "created": 1677610602,
  "owned_by": "openai"
}
```

### Chat Completions

#### Create Chat Completion
```http
POST /v1/chat/completions
```

Creates a completion for the chat message.

**Request Body:**
```json
{
  "model": "gpt-5",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant."
    },
    {
      "role": "user", 
      "content": "Hello, how are you?"
    }
  ],
  "temperature": 0.7,
  "max_tokens": 1000,
  "stream": false,
  "top_p": 1.0,
  "frequency_penalty": 0.0,
  "presence_penalty": 0.0,
  "functions": [],
  "function_call": "auto"
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `model` | string | Yes | Model to use ("gpt-5", "gpt-4o", "gpt-4", "gpt-3.5-turbo") |
| `messages` | array | Yes | Array of message objects |
| `temperature` | number | No | Sampling temperature (0-2, default: 1) |
| `max_tokens` | integer | No | Maximum tokens to generate |
| `stream` | boolean | No | Whether to stream responses (default: false) |
| `top_p` | number | No | Nucleus sampling parameter (0-1, default: 1) |
| `frequency_penalty` | number | No | Frequency penalty (-2 to 2, default: 0) |
| `presence_penalty` | number | No | Presence penalty (-2 to 2, default: 0) |
| `functions` | array | No | Available functions for function calling |
| `function_call` | string/object | No | Controls function calling behavior |

**Advanced Parameters (GPT-5 specific):**
```json
{
  "model": "gpt-5",
  "messages": [...],
  "extra_body": {
    "reasoning_effort": "high",
    "reasoning_summary": "detailed"
  }
}
```

| Advanced Parameter | Type | Description |
|-------------------|------|-------------|
| `reasoning_effort` | string | Controls depth of reasoning ("low", "medium", "high") |
| `reasoning_summary` | string | Format of reasoning output ("none", "concise", "auto", "detailed") |

**Response (Non-streaming):**
```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "gpt-5",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! I'm doing well, thank you for asking. How can I assist you today?"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 20,
    "completion_tokens": 18,
    "total_tokens": 38
  }
}
```

**Response (Streaming):**
```
data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-5","choices":[{"delta":{"content":"Hello"},"index":0,"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-5","choices":[{"delta":{"content":"!"},"index":0,"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-5","choices":[{"delta":{},"index":0,"finish_reason":"stop"}]}

data: [DONE]
```

## Message Formats

### System Message
```json
{
  "role": "system",
  "content": "You are a helpful assistant with expertise in software development."
}
```

### User Message
```json
{
  "role": "user",
  "content": "Write a Python function to calculate the Fibonacci sequence."
}
```

### Assistant Message
```json
{
  "role": "assistant", 
  "content": "Here's a Python function to calculate the Fibonacci sequence:\n\n```python\ndef fibonacci(n):\n    if n <= 1:\n        return n\n    return fibonacci(n-1) + fibonacci(n-2)\n```"
}
```

### Function Call Message
```json
{
  "role": "assistant",
  "content": null,
  "function_call": {
    "name": "get_weather",
    "arguments": "{\"location\": \"San Francisco\"}"
  }
}
```

### Function Response Message
```json
{
  "role": "function",
  "name": "get_weather", 
  "content": "{\"temperature\": \"72F\", \"condition\": \"sunny\"}"
}
```

## Function Calling

### Define Functions
```json
{
  "model": "gpt-5",
  "messages": [...],
  "functions": [
    {
      "name": "get_weather",
      "description": "Get the current weather for a location",
      "parameters": {
        "type": "object",
        "properties": {
          "location": {
            "type": "string",
            "description": "The city name"
          },
          "unit": {
            "type": "string",
            "enum": ["celsius", "fahrenheit"],
            "description": "Temperature unit"
          }
        },
        "required": ["location"]
      }
    }
  ],
  "function_call": "auto"
}
```

### Function Call Control

| Value | Description |
|-------|-------------|
| `"auto"` | Model decides whether to call functions |
| `"none"` | Model will not call any functions |
| `{"name": "function_name"}` | Forces the model to call specific function |

## Vision Support (GPT-4o)

### Image Input
```json
{
  "model": "gpt-4o",
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "text",
          "text": "What's in this image?"
        },
        {
          "type": "image_url", 
          "image_url": {
            "url": "https://example.com/image.jpg"
          }
        }
      ]
    }
  ]
}
```

### Base64 Image
```json
{
  "type": "image_url",
  "image_url": {
    "url": "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQ..."
  }
}
```

## Error Handling

### Error Response Format
```json
{
  "error": {
    "message": "Invalid request: missing required parameter 'messages'",
    "type": "invalid_request_error",
    "code": "missing_parameter"
  }
}
```

### Common Error Codes

| HTTP Status | Error Type | Description |
|-------------|------------|-------------|
| 400 | `invalid_request_error` | Invalid request format or parameters |
| 401 | `authentication_error` | Authentication required or failed |
| 403 | `permission_error` | Insufficient permissions |
| 429 | `rate_limit_error` | Request rate limit exceeded |
| 500 | `api_error` | Internal server error |
| 502 | `api_connection_error` | Error connecting to ChatGPT |
| 503 | `service_unavailable` | Service temporarily unavailable |

### Retry Strategy

For temporary errors (429, 502, 503), implement exponential backoff:

```python
import time
import random

def api_call_with_retry(func, max_retries=3):
    for attempt in range(max_retries):
        try:
            return func()
        except APIError as e:
            if e.status_code in [429, 502, 503] and attempt < max_retries - 1:
                delay = (2 ** attempt) + random.uniform(0, 1)
                time.sleep(delay)
                continue
            raise
```

## Rate Limiting

MindLink respects ChatGPT's rate limits and provides appropriate headers:

```http
X-RateLimit-Limit: 5000
X-RateLimit-Remaining: 4999
X-RateLimit-Reset: 1677652400
```

## Health Check Endpoint

### Check API Health
```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "1.0.0",
  "services": {
    "auth": "authenticated",
    "server": "running",
    "tunnel": "connected"
  }
}
```

## WebSocket Support

### Real-time Streaming
```javascript
const ws = new WebSocket('ws://localhost:3001/v1/realtime');

ws.onopen = function() {
    ws.send(JSON.stringify({
        type: 'session.create',
        session: {
            model: 'gpt-5',
            voice: 'alloy'
        }
    }));
};

ws.onmessage = function(event) {
    const data = JSON.parse(event.data);
    console.log('Received:', data);
};
```

## SDK Examples

### Python (OpenAI SDK)
```python
from openai import OpenAI

# Point to your MindLink instance
client = OpenAI(
    base_url="https://your-tunnel.trycloudflare.com/v1",
    api_key="any-key"  # Ignored, but required by SDK
)

# Standard chat completion
response = client.chat.completions.create(
    model="gpt-5",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello!"}
    ],
    temperature=0.7,
    max_tokens=1000,
    stream=True
)

# Stream the response
for chunk in response:
    if chunk.choices[0].delta.content:
        print(chunk.choices[0].delta.content, end="")
```

### Node.js
```javascript
import OpenAI from 'openai';

const openai = new OpenAI({
    baseURL: 'https://your-tunnel.trycloudflare.com/v1',
    apiKey: 'any-key' // Ignored
});

async function chatCompletion() {
    const response = await openai.chat.completions.create({
        model: 'gpt-5',
        messages: [
            { role: 'system', content: 'You are a helpful assistant.' },
            { role: 'user', content: 'Write a haiku about programming.' }
        ],
        temperature: 0.8
    });
    
    console.log(response.choices[0].message.content);
}

chatCompletion();
```

### cURL Examples

#### Basic Chat Completion
```bash
curl "https://your-tunnel.trycloudflare.com/v1/chat/completions" \
  -H "Authorization: Bearer any-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-5",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  }'
```

#### Streaming Response
```bash
curl "https://your-tunnel.trycloudflare.com/v1/chat/completions" \
  -H "Authorization: Bearer any-key" \
  -H "Content-Type: application/json" \
  -N \
  -d '{
    "model": "gpt-5", 
    "messages": [
      {"role": "user", "content": "Count to 10"}
    ],
    "stream": true
  }'
```

#### Function Calling
```bash
curl "https://your-tunnel.trycloudflare.com/v1/chat/completions" \
  -H "Authorization: Bearer any-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-5",
    "messages": [
      {"role": "user", "content": "What is the weather like in San Francisco?"}
    ],
    "functions": [
      {
        "name": "get_weather",
        "description": "Get weather information",
        "parameters": {
          "type": "object",
          "properties": {
            "location": {"type": "string"}
          },
          "required": ["location"]
        }
      }
    ],
    "function_call": "auto"
  }'
```

## Best Practices

### Performance Optimization
- Use streaming for long responses
- Implement client-side caching for repeated requests
- Set appropriate `max_tokens` to avoid unnecessary generation
- Use function calling for structured data extraction

### Error Handling
- Always check response status codes
- Implement retry logic with exponential backoff
- Handle network timeouts gracefully
- Log errors with sufficient context for debugging

### Security
- Use HTTPS endpoints in production
- Validate all inputs before sending to API
- Implement request/response logging for audit trails
- Monitor for unusual usage patterns

### Rate Limiting
- Respect the rate limits indicated in response headers
- Implement queuing for high-volume applications
- Consider using multiple MindLink instances for scaling
- Monitor usage to stay within ChatGPT subscription limits

This API reference ensures you can integrate MindLink seamlessly with any OpenAI-compatible application while leveraging the full power of your ChatGPT Plus/Pro subscription.