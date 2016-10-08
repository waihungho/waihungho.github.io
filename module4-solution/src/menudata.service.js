(function () {
'use strict';

angular.module('Data')
.constant('ApiBasePath', "https://davids-restaurant.herokuapp.com")
.service('MenudataService', MenudataService);


MenudataService.$inject = ['$http', 'ApiBasePath']
function MenudataService($http, ApiBasePath) {
  var service = this;

  service.getAllCategories = function () {
    var response = $http({
      method: "GET",
      url: (ApiBasePath + "/categories.json")
    });
    console.log("service.getAllCategories", response);
    return response;
  };

  service.getItemsForCategory = function (categoryShortName) {
    var response = $http({
      method: "GET",
      url: (ApiBasePath + "/menu_items.json"),
      params: {
        category: categoryShortName
      }
    });
    console.log("service.getItemsForCategory", response);
    return response;
  };
}

})();
