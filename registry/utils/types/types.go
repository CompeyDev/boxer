package types

import "github.com/gin-gonic/gin"

type TRouteManager struct {
	RoutesCollection map[string]func(*gin.Context)
	AddRoute         func(string, func(*gin.Context))
	PopulateSelf     func()
	main             func()
}

type TServer struct {
	InstanceId string
	Uptime     int32
	Instance   *gin.Engine
}

func (srv TServer) Run() {
	srv.Instance.Run()
}

func (srv TServer) Register(method string, route string, handler func(*gin.Context)) {
	switch method {
	case "GET":
		srv.Instance.GET(route, handler)
	case "POST":
		srv.Instance.POST(route, handler)
	}
}
