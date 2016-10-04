(function () {
'use strict';

angular.module('NarrowItDownApp', [])
.controller('NarrowItDownController', NarrowItDownController)
.service('MenuSearchService', MenuSearchService)
.constant('ApiBasePath', "https://davids-restaurant.herokuapp.com")
.directive('foundItems', FoundItemsDirective);

function FoundItemsDirective() {
  var ddo = {
    templateUrl: 'foundItems.html',
    scope: {
      found: '<',
      onRemove: '&'
    },
    controller: NarrowItDownController,
    controllerAs: 'list',
    bindToController: true,
  };
  return ddo;
}


NarrowItDownController.$inject = ['$scope','MenuSearchService'];
function NarrowItDownController($scope, MenuSearchService) {
  var list = this;
  list.title = 'Victor';

  list.getMenu = function () {
    var promise = MenuSearchService.getMatchedMenuItems();
    promise.then(function (response) {
      list.found = response.data.menu_items.filter(function (el) {
        return  ( el.description.indexOf($scope.searchTerm) !== -1 );
      });
      //console.log(list.found);
    })
    .catch(function (error) {
      console.log("Something went terribly wrong.");
    });
  };

  list.removeItem = function (itemIndex) {
    console.log('Remove',itemIndex);
    list.found.splice(itemIndex, 1);
  };

}


MenuSearchService.$inject = ['$http', 'ApiBasePath']
function MenuSearchService($http, ApiBasePath) {
  var service = this;

  service.getMatchedMenuItems = function () {
    var response = $http({
      method: "GET",
      url: (ApiBasePath + "/menu_items.json")
    });
    return response;
  };
}

})();
