package main

/*
#cgo LDFLAGS: -L./target/release -ldeobfuscator -ldl -lm
#include <stdlib.h>
extern char* deobfuscate_code(char* code, char* language);
extern void free_string(char* ptr);
*/
import "C"

import (
    "archive/zip"
    "bytes"
    "encoding/json"
    "fmt"
    "io"
    "log"
    "net/http"
    "os"
    "os/signal"
    "strings"
    "syscall"
    "time"
    "unsafe"

    "github.com/bwmarrin/discordgo"
    "github.com/gin-gonic/gin"
)

type DeobfuscateRequest struct {
    Code            string `json:"code"`
    Language        string `json:"language"`
    ObfuscationType string `json:"obfuscation_type"`
    URL             string `json:"url"`
}

type DeobfuscateResponse struct {
    OriginalCode           string   `json:"original_code"`
    ObfuscationType        string   `json:"obfuscation_type"`
    Confidence             float64  `json:"confidence"`
    ExecutionTimeMS        uint64   `json:"execution_time_ms"`
    TransformationsApplied []string `json:"transformations_applied"`
    DetectedLanguage       string   `json:"detected_language"`
}

type OpenRouterRequest struct {
    Model       string              `json:"model"`
    Messages    []OpenRouterMessage `json:"messages"`
    Temperature float64             `json:"temperature"`
    MaxTokens   int                 `json:"max_tokens"`
}

type OpenRouterMessage struct {
    Role    string `json:"role"`
    Content string `json:"content"`
}

type OpenRouterResponse struct {
    Choices []struct {
        Message struct {
            Content string `json:"content"`
        } `json:"message"`
    } `json:"choices"`
}

func main() {
    go startAPIServer()
    startDiscordBot()
}

func startAPIServer() {
    router := gin.Default()

    router.POST("/api/deobfuscate", func(c *gin.Context) {
        var req DeobfuscateRequest

        if err := c.BindJSON(&req); err != nil {
            c.JSON(400, gin.H{"error": err.Error()})
            return
        }

        var code string

        if req.URL != "" {
            fetched, err := fetchFromURL(req.URL)
            if err != nil {
                c.JSON(400, gin.H{"error": "failed to fetch URL: " + err.Error()})
                return
            }
            code = fetched
        } else if req.Code != "" {
            code = req.Code
        } else {
            c.JSON(400, gin.H{"error": "code or url is required"})
            return
        }

        result := deobfuscateCode(code, req.Language, req.ObfuscationType)

        zipData := createZip(result)
        filename := fmt.Sprintf("deobfuscated_%d.zip", time.Now().Unix())

        c.Header("Content-Type", "application/zip")
        c.Header("Content-Disposition", fmt.Sprintf("attachment; filename=%s", filename))
        c.Data(200, "application/zip", zipData)
    })

    router.POST("/api/deobfuscate/file", func(c *gin.Context) {
        file, err := c.FormFile("file")
        if err != nil {
            c.JSON(400, gin.H{"error": "file is required"})
            return
        }

        f, err := file.Open()
        if err != nil {
            c.JSON(400, gin.H{"error": err.Error()})
            return
        }
        defer f.Close()

        content, err := io.ReadAll(f)
        if err != nil {
            c.JSON(400, gin.H{"error": err.Error()})
            return
        }

        language := c.PostForm("language")
        obfuscationType := c.PostForm("obfuscation_type")

        result := deobfuscateCode(string(content), language, obfuscationType)

        zipData := createZip(result)
        filename := fmt.Sprintf("deobfuscated_%d.zip", time.Now().Unix())

        c.Header("Content-Type", "application/zip")
        c.Header("Content-Disposition", fmt.Sprintf("attachment; filename=%s", filename))
        c.Data(200, "application/zip", zipData)
    })

    router.GET("/api/health", func(c *gin.Context) {
        c.JSON(200, gin.H{"status": "ok"})
    })

    port := os.Getenv("PORT")
    if port == "" {
        port = "8080"
    }
    router.Run(":" + port)
}

func fetchFromURL(url string) (string, error) {
    client := &http.Client{
        Timeout: 30 * time.Second,
    }

    req, err := http.NewRequest("GET", url, nil)
    if err != nil {
        return "", err
    }

    req.Header.Set("User-Agent", "Mozilla/5.0")
    req.Header.Set("Accept", "*/*")

    resp, err := client.Do(req)
    if err != nil {
        return "", err
    }
    defer resp.Body.Close()

    if resp.StatusCode != 200 {
        return "", fmt.Errorf("status: %d", resp.StatusCode)
    }

    content, err := io.ReadAll(resp.Body)
    if err != nil {
        return "", err
    }

    return string(content), nil
}

func startDiscordBot() {
    token := os.Getenv("DISCORD_BOT_TOKEN")
    if token == "" {
        log.Println("DISCORD_BOT_TOKEN not set, skipping Discord Bot")
        select {}
    }

    dg, err := discordgo.New("Bot " + token)
    if err != nil {
        log.Fatal(err)
    }

    dg.AddHandler(messageCreate)
    dg.Identify.Intents = discordgo.IntentsGuildMessages | discordgo.IntentsDirectMessages | discordgo.IntentsMessageContent

    if err := dg.Open(); err != nil {
        log.Fatal(err)
    }

    fmt.Println("Discord Bot running")

    sc := make(chan os.Signal, 1)
    signal.Notify(sc, syscall.SIGINT, syscall.SIGTERM, os.Interrupt)
    <-sc

    dg.Close()
}

func messageCreate(s *discordgo.Session, m *discordgo.MessageCreate) {
    if m.Author.ID == s.State.User.ID {
        return
    }

    if strings.HasPrefix(m.Content, "/raw ") {
        handleRawCommand(s, m)
        return
    }

    if strings.HasPrefix(m.Content, "!deobfuscate") {
        handleDeobfuscateCommand(s, m)
        return
    }

    if strings.HasPrefix(m.Content, "!help") || strings.HasPrefix(m.Content, "/help") {
        s.ChannelMessageSend(m.ChannelID, "使い方:\n/raw <URL>\n!deobfuscate <URL>\n!deobfuscate <コード>\nファイルを添付して !deobfuscate")
    }
}

func handleRawCommand(s *discordgo.Session, m *discordgo.MessageCreate) {
    url := strings.TrimPrefix(m.Content, "/raw ")
    url = strings.TrimSpace(url)

    if url == "" {
        s.ChannelMessageSend(m.ChannelID, "URL not found")
        return
    }

    content, err := fetchFromURL(url)
    if err != nil {
        s.ChannelMessageSend(m.ChannelID, "failed to fetch URL: "+err.Error())
        return
    }

    result := deobfuscateCode(content, "", "")

    zipData := createZip(result)

    s.ChannelMessageSendComplex(m.ChannelID, &discordgo.MessageSend{
        Content: fmt.Sprintf("type: %s | lang: %s | confidence: %.1f%% | time: %dms",
            result.ObfuscationType, result.DetectedLanguage, result.Confidence*100, result.ExecutionTimeMS),
        Files: []*discordgo.File{
            {
                Name:   "deobfuscated.zip",
                Reader: bytes.NewReader(zipData),
            },
        },
    })
}

func handleDeobfuscateCommand(s *discordgo.Session, m *discordgo.MessageCreate) {
    var code string
    var language string
    var obfuscationType string

    if len(m.Attachments) > 0 {
        attachment := m.Attachments[0]
        content, err := fetchFromURL(attachment.URL)
        if err != nil {
            s.ChannelMessageSend(m.ChannelID, "failed to fetch attachment: "+err.Error())
            return
        }
        code = content
    }

    if code == "" {
        code = extractCode(m.Content)
    }

    if code == "" {
        url := extractURL(m.Content)
        if url != "" {
            content, err := fetchFromURL(url)
            if err != nil {
                s.ChannelMessageSend(m.ChannelID, "failed to fetch URL: "+err.Error())
                return
            }
            code = content
        }
    }

    if code == "" {
        s.ChannelMessageSend(m.ChannelID, "code, url, or file not found")
        return
    }

    result := deobfuscateCode(code, language, obfuscationType)

    zipData := createZip(result)

    s.ChannelMessageSendComplex(m.ChannelID, &discordgo.MessageSend{
        Content: fmt.Sprintf("type: %s | lang: %s | confidence: %.1f%% | time: %dms",
            result.ObfuscationType, result.DetectedLanguage, result.Confidence*100, result.ExecutionTimeMS),
        Files: []*discordgo.File{
            {
                Name:   "deobfuscated.zip",
                Reader: bytes.NewReader(zipData),
            },
        },
    })
}

func extractURL(content string) string {
    words := strings.Fields(content)
    for _, word := range words {
        if strings.HasPrefix(word, "http://") || strings.HasPrefix(word, "https://") {
            return word
        }
    }
    return ""
}

func createZip(result DeobfuscateResponse) []byte {
    buf := new(bytes.Buffer)
    w := zip.NewWriter(buf)

    f, _ := w.Create("deobfuscated.txt")
    f.Write([]byte(result.OriginalCode))

    info, _ := w.Create("info.json")
    infoData, _ := json.MarshalIndent(result, "", "  ")
    info.Write(infoData)

    w.Close()
    return buf.Bytes()
}

func extractCode(content string) string {
    if strings.Contains(content, "```") {
        parts := strings.Split(content, "```")
        if len(parts) >= 2 {
            code := parts[1]
            if idx := strings.Index(code, "\n"); idx != -1 {
                code = code[idx+1:]
            }
            return code
        }
    }

    prefix := "!deobfuscate "
    if strings.HasPrefix(content, prefix) {
        remainder := strings.TrimPrefix(content, prefix)
        if !strings.HasPrefix(remainder, "http://") && !strings.HasPrefix(remainder, "https://") {
            return remainder
        }
    }

    return ""
}

func deobfuscateWithAI(code string) (string, error) {
    apiKey := os.Getenv("OPENROUTER_API_KEY")
    if apiKey == "" {
        return "", fmt.Errorf("OPENROUTER_API_KEY not set")
    }

    log.Println("AI deobfuscation started")

    reqBody := OpenRouterRequest{
        Model: "nvidia/nemotron-3.5-lightning:free",
        Messages: []OpenRouterMessage{
            {
                Role: "system",
                Content: "You are a code deobfuscator. Decode and restore the original code. Return only the deobfuscated code without any explanation or markdown.",
            },
            {
                Role: "user",
                Content: fmt.Sprintf("Deobfuscate this code:\n\n%s", code),
            },
        },
        Temperature: 0.1,
        MaxTokens:   4000,
    }

    jsonData, _ := json.Marshal(reqBody)

    req, _ := http.NewRequest("POST", "https://openrouter.ai/api/v1/chat/completions", bytes.NewBuffer(jsonData))
    req.Header.Set("Content-Type", "application/json")
    req.Header.Set("Authorization", "Bearer "+apiKey)

    client := &http.Client{Timeout: 120 * time.Second}
    resp, err := client.Do(req)
    if err != nil {
        log.Println("AI request error:", err)
        return "", err
    }
    defer resp.Body.Close()

    body, _ := io.ReadAll(resp.Body)
    log.Println("AI response status:", resp.StatusCode)
    log.Println("AI response body:", string(body))

    var openRouterResp OpenRouterResponse
    json.Unmarshal(body, &openRouterResp)

    if len(openRouterResp.Choices) > 0 {
        log.Println("AI success")
        return openRouterResp.Choices[0].Message.Content, nil
    }

    return "", fmt.Errorf("no response from OpenRouter")
}

func deobfuscateCode(code string, language string, obfuscationType string) DeobfuscateResponse {
    cCode := C.CString(code)
    cLang := C.CString(language)

    defer C.free(unsafe.Pointer(cCode))
    defer C.free(unsafe.Pointer(cLang))

    result := C.deobfuscate_code(cCode, cLang)
    defer C.free_string(result)

    jsonStr := C.GoString(result)

    var response DeobfuscateResponse
    json.Unmarshal([]byte(jsonStr), &response)

    if os.Getenv("OPENROUTER_API_KEY") != "" {
        aiResult, err := deobfuscateWithAI(code)
        if err != nil {
            log.Println("AI error:", err)
        }
        if err == nil && aiResult != "" && aiResult != code {
            log.Println("AI result applied")
            response.OriginalCode = aiResult
            response.TransformationsApplied = append(response.TransformationsApplied, "ai_deobfuscate")
        }
    } else {
        log.Println("OPENROUTER_API_KEY not set")
    }

    return response
}
