(function () {
'use strict';

angular.module('ShoppingListCheckOff', [])
.controller('ShoppingController', ShoppingController)
.controller('ToBuyShoppingController', ToBuyShoppingController)
.controller('AlreadyBoughtShoppingController',AlreadyBoughtShoppingController)
.service("ShoppingListCheckOffService", ShoppingListCheckOffService);
//.factory('ShoppingListCheckOffFactory', ShoppingListCheckOffFactory);
// .config(Config);
// .provider('ShoppingListCheckOffService', ShoppingListCheckOffServiceProvider)

// Config.$inject = ['ShoppingListCheckOffService'];
// function Config(ShoppingListCheckOffService) {
//   ShoppingListCheckOffService.setItems(ShoppingListCheckOffService.getDefaultToBuyItems());
// }


ShoppingController.$inject = ['$scope','ShoppingListCheckOffService'];
function ShoppingController($scope, ShoppingListCheckOffService) {
  var list = this;
  const noBuyMsg = 'Everything is bought!';
  const noBoughtMsg = 'Nothing bought yet';
  list.boughtMessage = noBoughtMsg;
  $scope.showMsg = function () {
    if (ShoppingListCheckOffService.getBoughtItems().length == 0) {
      list.boughtMessage = noBoughtMsg;
    } else {
      list.boughtMessage = '';
    }
    if (ShoppingListCheckOffService.getBuyItems().length == 0) {
      list.toBuyMessage= noBuyMsg;
    } else {
      list.toBuyMessage= ' ';
    }
  };
}



ToBuyShoppingController.$inject = ['$scope','ShoppingListCheckOffService'];
function ToBuyShoppingController($scope, ShoppingListCheckOffService) {
  var list = this;
  //var thisService = ShoppingListCheckOffFactory();
  //console.log(thisService);
  list.items = ShoppingListCheckOffService.getDefaultToBuyItems();
  list.removeItem = function (itemIndex) {
    ShoppingListCheckOffService.removeBuyItem(itemIndex);
    $scope.showMsg();
  };
}

AlreadyBoughtShoppingController.$inject = ['$scope','ShoppingListCheckOffService'];
function AlreadyBoughtShoppingController($scope, ShoppingListCheckOffService) {
  var list = this;
  list.items = ShoppingListCheckOffService.getBoughtItems();
  list.removeItem = function (itemIndex) {
    ShoppingListCheckOffService.removeBoughtItem(itemIndex);
    $scope.showMsg();
  };
}




// If not specified, maxItems assumed unlimited
function ShoppingListCheckOffService() {
  var service = this;

  // List of shopping items
  var boughtItems = [];

  var defaultToBuyItems = [
                            { name: "cookies", quantity: 10 },
                            { name: "apples", quantity: 999 },
                            { name: "iPads", quantity: 10000 },
                            { name: "iPhone7+", quantity: 1 },
                            { name: "Lenovo", quantity: 5 }                          ];

  var toBuyItems = defaultToBuyItems;
//  service.setItems = function (items){
//    toBuyItems=items;
//  };
  service.getItService = function (){
    return service;
  }
  service.addBuyItem = function (itemName, quantity) {
    // if ((maxItems === undefined) ||
    //     (maxItems !== undefined) && (items.length < maxItems)) {
      var item = {
        name: itemName,
        quantity: quantity
      };
      toBuyItems.push(item);
    // }
    // else {
    //   throw new Error("Max items (" + maxItems + ") reached.");
    // }
  };

  service.removeBuyItem = function (itemIndex) {
    boughtItems.push(toBuyItems[itemIndex]);
    toBuyItems.splice(itemIndex, 1);
  };

  service.getBuyItems = function () {
    return toBuyItems;
  };
  service.getDefaultToBuyItems = function () {
    return defaultToBuyItems;
  };



  service.addBoughtItem = function (itemName, quantity) {
    // if ((maxItems === undefined) ||
    //     (maxItems !== undefined) && (items.length < maxItems)) {
      var item = {
        name: itemName,
        quantity: quantity
      };
      boughtItems.push(item);
    // }
    // else {
    //   throw new Error("Max items (" + maxItems + ") reached.");
    // }
  };

  service.removeBoughtItem = function (itemIndex) {
    toBuyItems.push(boughtItems[itemIndex]);
    boughtItems.splice(itemIndex, 1);
  };

  service.getBoughtItems = function () {
    return boughtItems;
  };

}

// function ShoppingListCheckOffFactory() {
//   var factory = function () {
//     return ShoppingListCheckOffService;
//   };
//
//   return factory;
// }



// function ShoppingListCheckOffServiceProvider() {
//   var provider = this;
//
//   provider.defaults = {
//     maxItems: 10
//   };
//
//   provider.$get = function () {
//     var shoppingList = new ShoppingListCheckOffService(provider.defaults.maxItems);
//
//     return shoppingList;
//   };
// }

})();
