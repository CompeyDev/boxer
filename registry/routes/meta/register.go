package meta

import (
	"context"

	"github.com/CompeyDev/boxer/registry/constants"
	"github.com/CompeyDev/boxer/registry/prisma/db"
	"github.com/CompeyDev/boxer/registry/utils/logger"
	types "github.com/CompeyDev/boxer/registry/utils/types"
	"github.com/gin-gonic/gin"
)

func RegisterFetcher(routeManager types.TRouteManager) {
	routeManager.AddRoute("/api/meta/:scope/:pkg::GET", func(ctx *gin.Context) {
		scope, scopeGetSuccess := ctx.Params.Get("scope")
		pkgName, pkgNameGetSuccess := ctx.Params.Get("pkg")

		if !scopeGetSuccess || !pkgNameGetSuccess {
			logger.Error("  API  ", "meta: failed to fetch scope OR pkg params from route addr")
		}

		prismaClient := constants.Get("prismaClient").(*db.PrismaClient)

		packageMeta, fetchErr := prismaClient.Package.FindFirst(
			db.Package.Owner.Equals(scope),
			db.Package.Name.Equals(pkgName),
		).Exec(context.Background())

		if fetchErr != nil {
			logger.Error("  API  ", "meta::db: failed to fetch package meta details")

			if fetchErr.Error() == "ErrNotFound" {
				ctx.JSON(404, gin.H{
					"status":  404,
					"message": "package not found",
				})
			} else {
				ctx.JSON(502, gin.H{
					"status":  502,
					"message": "internal error",
				})
			}
		}

		println(packageMeta)
	})
}

func RegisterSetter(routeManager types.TRouteManager) {
	routeManager.AddRoute("/api/meta/:scope/:pkg::POST", func(ctx *gin.Context) {
		scope, scopeGetSuccess := ctx.Params.Get("scope")
		pkgName, pkgNameGetSuccess := ctx.Params.Get("pkg")

		type ReqBody struct {
			LatestVersion string `json:"latest_version"`
		}

		body := ReqBody{}

		if bindJsonErr := ctx.BindJSON(&body); bindJsonErr != nil {
			logger.Error("  API  ", "meta: failed to get request body")

			ctx.JSON(502, gin.H{
				"status":  502,
				"message": "internal error",
			})

			return
		}
		// TODO: check if it doesn't exist already

		if !scopeGetSuccess || !pkgNameGetSuccess {
			logger.Error("  API  ", "meta: failed to fetch scope OR pkg params from route addr")

			ctx.JSON(502, gin.H{
				"status":  502,
				"message": "internal error",
			})

			return
		}

		prismaClient := constants.Get("prismaClient").(*db.PrismaClient)

		newPackageMeta, setErr := prismaClient.Package.CreateOne(
			db.Package.Owner.Set(scope),
			db.Package.Name.Set(pkgName),
		).Exec(context.Background())

		if setErr != nil {
			logger.Error("  API  ", "meta::db: failed to set package meta details")

			ctx.JSON(502, gin.H{
				"status":  502,
				"message": "internal error",
			})

			return
		}

		packageMetaModel, fetchErr := prismaClient.PackageVersion.CreateOne(
			db.PackageVersion.Version.Set(body.LatestVersion),
			db.PackageVersion.AssocPackage.Link(
				db.Package.ID.Equals(newPackageMeta.ID),
			),
		).Exec(context.Background())

		if fetchErr != nil {
			logger.Error("  API  ", "meta::db: failed to fetch package meta details")

			ctx.JSON(502, gin.H{
				"status":  502,
				"message": "internal error",
			})

			return
		}

		println(packageMetaModel.AssocPackageID)

	})
}
