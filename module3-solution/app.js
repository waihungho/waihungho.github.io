(function () {
'use strict';

angular.module('NarrowItDownApp', [])
.controller('NarrowItDownController', NarrowItDownController)
.service('MenuSearchService', MenuSearchService)
.constant('ApiBasePath', "https://davids-restaurant.herokuapp.com");

NarrowItDownController.$inject = ['$scope','MenuSearchService'];
function NarrowItDownController($scope, MenuSearchService) {
  var ctrl = this;
  ctrl.getMenu = function (shortName) {
    var promise = MenuSearchService.getMatchedMenuItems();
    promise.then(function (response) {
      ctrl.found = response.data.menu_items.filter(function (el) {
        return  ( el.description.indexOf($scope.searchTerm) !== -1 );
      });
    })
    .catch(function (error) {
      console.log("Something went terribly wrong.");
    });
  };
}


MenuSearchService.$inject = ['$http', 'ApiBasePath']
function MenuSearchService($http, ApiBasePath) {
  var service = this;

  service.getMatchedMenuItems = function () {
    var response = $http({
      method: "GET",
      url: (ApiBasePath + "/menu_items.json")
      // url: (ApiBasePath + "/categories.json")
    });
    return response;
  };
}

})();
