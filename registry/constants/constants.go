package constants

var Constants = map[string]any{
	"apiVersion": "v2",
	"port":       8080,
	"requestBodySchema": map[string]any{
		"/heartbeat": []string{},
	},
}

func RegisterOrSet(constant string, value any) {
	Constants[constant] = value
}

func Get(constant string) any {
	return Constants[constant]
}
