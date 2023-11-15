document.addEventListener('DOMContentLoaded', function() {
  // Получаем список пользователей
  const users = document.querySelectorAll('.chat-list li');
  users.forEach(user => {
      // Добавляем обработчик клика на каждого пользователя
      user.addEventListener('click', function(){
          // Убираем выделение у всех пользователей
          users.forEach(u => u.classList.remove('active'));
          // Добавляем выделение выбранному пользователю
          this.classList.add('active');

          // Получаем имя выбранного пользователя и обновляем заголовок чата
          const userName = this.querySelector('.name').textContent;
          const chatHeader = document.querySelector('.chat-header h6');
          chatHeader.textContent = userName;
      });
  });

  // Получаем элемент для ввода поискового запроса
  const searchInput = document.getElementById('searchInput');
  // Добавляем обработчик события ввода для поиска
  searchInput.addEventListener('input', function () {
      // Получаем поисковый запрос и приводим его к нижнему регистру
      const searchTerm = this.value.toLowerCase();

      // Проходим по каждому пользователю
      users.forEach(user => {
          // Получаем имя пользователя и приводим его к нижнему регистру
          const userName = user.querySelector('.name').textContent.toLowerCase();

          // Проверяем, содержит ли имя пользователя поисковый запрос
          if (userName.includes(searchTerm)) {
              user.style.display = 'block';
          } else {
              user.style.display = 'none';
          }
      });
  });

  // Получаем элементы для ввода сообщения, отображения сообщений и кнопки "Отправить"
  const messageInput = document.getElementById('messageInput');
  const messageDisplay = document.getElementById('messageDisplay');
  const sendButton = document.getElementById('sendButton');

  // Добавляем обработчик для проверки ввода сообщения и активации/деактивации кнопки "Отправить"
  messageInput.addEventListener('input', function () {
      const messageText = this.value.trim(); // Убираем начальные и конечные пробелы

      // Если поле ввода содержит текст, делаем кнопку "Отправить" активной, иначе делаем ее неактивной
      if (messageText !== '') {
          sendButton.disabled = false; // Активируем кнопку "Отправить"
      } else {
          sendButton.disabled = true; // Деактивируем кнопку "Отправить"
      }
  });

  // Добавляем обработчик для отправки сообщения
  sendButton.addEventListener('click', function () {
      const messageText = messageInput.value.trim();

      if (messageText !== '') {
          // Создаем новый элемент сообщения
          const messageElement = document.createElement('div');
          messageElement.classList.add('message');
          messageElement.textContent = messageText;

          // Добавляем элемент сообщения в контейнер для отображения сообщений
          messageDisplay.appendChild(messageElement);

          // Очищаем поле ввода сообщения
          messageInput.value = '';

          // Деактивируем кнопку "Отправить" после отправки сообщения
          sendButton.disabled = true;
      }
  });
});
