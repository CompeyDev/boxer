package main

import (
	"github.com/CompeyDev/boxer/registry/constants"
	types "github.com/CompeyDev/boxer/registry/utils/types"
	"github.com/gin-gonic/gin"
)

type Server = types.TServer

func NewServer() Server {

	return Server{
		InstanceId: constants.Get("instanceId").(string),
		Uptime:     0,
		Instance:   gin.New(),
	}
}
