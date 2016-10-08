(function () {
'use strict';

angular.module('MenuApp')
.config(RoutesConfig);

RoutesConfig.$inject = ['$stateProvider', '$urlRouterProvider'];
function RoutesConfig($stateProvider, $urlRouterProvider) {

  // Redirect to home page if no other URL matches
  $urlRouterProvider.otherwise('/');

  // *** Set up UI states ***
  $stateProvider

  // Home page
  .state('home', {
    url: '/',
    templateUrl: 'src/template/home.template.html'
  })

  .state('categories', {
    url: '/categories',
    templateUrl: 'src/template/categories.template.html',
    controller: 'MenuAppController as menuCtrl',
    resolve: {
      items: ['MenudataService', function (MenudataService) {
        return MenudataService.getAllCategories();
      }]
    }
  })

  .state('categories.menuItems', {
    url: '/menu-items/{catId}',
    templateUrl: 'src/template/items.template.html',
    controller: 'ItemsController as itemCtrl',
    resolve: {
      items: ['$stateParams', 'MenudataService',
            function ($stateParams, MenudataService) {
              return MenudataService.getItemsForCategory($stateParams.catId);
            }]
    }
  });

}

})();
