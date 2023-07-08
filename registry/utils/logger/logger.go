package logger

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/CompeyDev/boxer/registry/constants"
	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/gookit/color"
)

func Info(logType string, msg any) {
	color.BgGreen.Print(
		color.Bold.Sprint(logType),
	)

	fmt.Println(fmt.Sprintf("  %s", msg))
}

func Error(logType string, msg any) {
	color.BgRed.Print(
		color.Bold.Sprint(logType),
	)

	fmt.Println(fmt.Sprintf("  %s", msg))
}

func Warn(logType string, msg any) {
	color.BgYellow.Print(
		color.Bold.Sprint(logType),
	)

	fmt.Println(fmt.Sprintf("  %s", msg))
}

func getClientType(userAgent string) string {
	var clientType string

	defer func() {
		if recover() != nil {
			// in case we cannot get our client type, we recover and set it to unknown
			Warn("  CORE  ", "Could not detect client type, assuming unknown")
		}
	}()

	if clientType == "" {
		return "Unknown"
	}

	clientType = strings.Split(userAgent, " ")[10]

	return clientType
}

func Middleware() gin.HandlerFunc {
	return func(ctx *gin.Context) {
		startBench := time.Now()
		ctx.Next()

		statusCode := ctx.Writer.Status()
		requestMethod := ctx.Request.Method
		requestRoute := ctx.Request.URL.Path
		requestId := fmt.Sprintf("request_%s", strings.Trim(uuid.New().URN(), "_rn:uuid:"))
		clientType := getClientType(ctx.Request.UserAgent())

		if requestMethod == "POST" {
			requestMethod = "  POST  "
		} else {
			requestMethod = fmt.Sprintf("  %s   ", requestMethod)
		}

		logMsg := fmt.Sprintf(
			"%s %s: %s, %s: %s -> %s",
			color.Blue.Sprintf("\033[4m%s\033", requestRoute), // route path
			color.Magenta.Sprint("requestID"),                 // "requestID"
			requestId,                                         // request ID
			color.HiGreen.Sprint("client"),                    // "client"
			clientType,                                        // client type
			fmt.Sprintf("%dms", time.Since(startBench).Microseconds()),
		)

		switch statusCode {
		case 200:
			Info(requestMethod, logMsg)

		default:
			Error(requestMethod, logMsg)
			var requestSchema []string
			requestSchemaTmp := constants.Get("requestBodySchema").(map[string]any)[requestRoute]

			if requestSchemaTmp != nil {
				requestSchema = requestSchemaTmp.([]string)
			} else if ctx.Request.Method != "GET" {
				Warn(
					"  CORE  ",
					fmt.Sprintf(
						"%s %s: %s %s",
						color.Blue.Sprintf("\033[4m%s\033", requestRoute),           // route path
						color.Magenta.Sprint("requestID"),                           // "requestID"
						requestId,                                                   // request ID
						color.Red.Sprint("No request body schema found for route."), // error msg
					),
				)
			}

			if ctx.Request.Method != "GET" {
				requestBody, requestBodyReadErr := ctx.GetRawData()

				if requestBodyReadErr != nil {
					Warn("  CORE  ", "Failed to read and validate request body.")
					return
				}

				var requestBodyContents map[string]*json.RawMessage

				if err := json.Unmarshal(requestBody, &requestBodyContents); err != nil {
					Warn(
						"  CORE  ",
						fmt.Sprintf(
							"%s %s: %s %s",
							color.Blue.Sprintf("\033[4m%s\033", requestRoute),             // route path
							color.Magenta.Sprint("requestID"),                             // "requestID"
							requestId,                                                     // request ID
							color.Red.Sprint("Failed to read and validate request body."), // error msg
						),
					)
					return
				}

				expectedValues := []string{}

				for requestKey := range requestBodyContents {
					for _, requestSchemaKey := range requestSchema {
						if requestKey != requestSchemaKey {
							expectedValues = append(expectedValues, requestSchemaKey)
						}
					}
				}

				Warn("  HINT  ",
					fmt.Sprintf(
						"expecting %s in request body but not provided",
						color.Bold.Sprint(
							strings.Split(
								strings.Trim(fmt.Sprint(expectedValues), "[]"),
								" ",
							)[0:len(expectedValues)],
						),
					),
				)
			}
		}
	}
}
